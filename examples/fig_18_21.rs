use std::{ffi::c_void, mem::MaybeUninit};

extern "C" fn sig_catch(signo: libc::c_int) {
    match signo {
        libc::SIGINT => println!("[SIGINT]"),
        libc::SIGQUIT => println!("[SIGQUIT]"),
        libc::SIGTERM => println!("[SIGTERM]"),
        _ => println!("[signal] {signo}"),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TtyState {
    Reset,
    Raw,
    CBreak,
}

struct TTyStateMachine {
    prev_termios: Option<libc::termios>,
    curr_state: TtyState,
}

impl Default for TTyStateMachine {
    fn default() -> Self {
        Self {
            prev_termios: Default::default(),
            curr_state: TtyState::Reset,
        }
    }
}

impl TTyStateMachine {
    fn push_state(&mut self, fd: libc::c_int, state: TtyState) {
        let mut curr_termios = {
            let mut p = MaybeUninit::uninit();
            unsafe {
                libc::tcgetattr(fd, p.as_mut_ptr());
                p.assume_init()
            }
        };
        self.prev_termios = Some(curr_termios.clone());
        match state {
            TtyState::Reset => {
                panic!("unexpected state");
            }
            TtyState::Raw => {
                assert_eq!(self.curr_state, TtyState::Reset);
                // echo off, canonical mode off, extended input processing off, signal chars off
                curr_termios.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);

                /*
                - no sigint on break, CR-to-NL off, input parity check off,
                - doesn't strip 8th bit on input
                - output flow control off
                 */
                curr_termios.c_iflag &=
                    !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);

                // clear size bits, parity checking off
                curr_termios.c_cflag &= !(libc::CSIZE | libc::PARENB);

                // set 8 bits/char.
                curr_termios.c_cflag |= libc::CS8;

                // output processing off
                curr_termios.c_oflag &= !(libc::OPOST);

                // case B: 1 byte at a time, no timer
                curr_termios.c_cc[libc::VMIN] = 1;
                curr_termios.c_cc[libc::VTIME] = 0;

                let r = unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &curr_termios) };
                assert_eq!(0, r);
            }
            TtyState::CBreak => {
                assert_eq!(self.curr_state, TtyState::Reset);
                /* put terminal into a cbreak mode */
                // echo off, canonical mode off
                curr_termios.c_lflag &= !(libc::ECHO | libc::ICANON);

                // case B: 1 byte at a time, no timer
                curr_termios.c_cc[libc::VMIN] = 1;
                curr_termios.c_cc[libc::VTIME] = 0;
                unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &curr_termios) };
            }
        }
        self.curr_state = state;
    }

    pub fn reset(&mut self, fd: libc::c_int) {
        if self.curr_state != TtyState::Reset {
            let prev_termios = self.prev_termios.take().unwrap();
            unsafe {
                libc::tcsetattr(fd, libc::TCSAFLUSH, &prev_termios);
            }
            self.curr_state = TtyState::Reset;
        }
    }
}

fn main() {
    let sig_handler = sig_catch as libc::sighandler_t;
    for sig in [libc::SIGINT, libc::SIGQUIT, libc::SIGTERM] {
        unsafe {
            assert_ne!(libc::SIG_ERR, libc::signal(sig, sig_handler));
        }
    }

    let mut sm = TTyStateMachine::default();
    sm.push_state(libc::STDIN_FILENO, TtyState::Raw);
    println!("enter raw mode characters, terminate with [DELETE]");
    let mut c: u8 = 0;
    while unsafe { libc::read(libc::STDIN_FILENO, &mut c as *mut u8 as *mut c_void, 1) } == 1 {
        c &= 255;
        if c == 0o177 {
            // 0o177 == ASCII DELETE (octal)
            break;
        }
        println!("{c:o}");
    }
    sm.reset(libc::STDIN_FILENO);
    sm.push_state(libc::STDIN_FILENO, TtyState::CBreak);
    println!("enter cbreak mode characters, terminate with [SIGINT]");
    while unsafe { libc::read(libc::STDIN_FILENO, &mut c as *mut u8 as *mut c_void, 1) } == 1 {
        c &= 255;
        println!("{c:o}");
    }
    sm.reset(libc::STDIN_FILENO);
}
