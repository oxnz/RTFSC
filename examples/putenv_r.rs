use std::ffi::CStr;
use std::io::Write;
use std::sync::atomic::{AtomicPtr, AtomicUsize};

fn main() {
    putenv_r(c"hello", c"world");
    putenv_r(c"hello2", c"world");
    printenv_r();
}

const MAX_ENV_COUNT: usize = 1024;
const MAX_ENV_STRLEN: usize = 128;
static mut ENV_COUNT: AtomicUsize = AtomicUsize::new(0);
static mut ENV_STORE: [[u8; MAX_ENV_STRLEN]; MAX_ENV_COUNT] = [[0; MAX_ENV_STRLEN]; MAX_ENV_COUNT];
static mut ENV_PTRS: [AtomicPtr<u8>; MAX_ENV_COUNT] =
    unsafe { std::mem::transmute([0usize; MAX_ENV_COUNT]) };

fn putenv_r(name: &CStr, value: &CStr) {
    if name.is_empty() || value.is_empty() {
        return;
    }
    let mut buf = Vec::with_capacity(MAX_ENV_STRLEN);
    buf.write(name.to_bytes()).unwrap();
    buf.write(b"=").unwrap();
    buf.write(value.to_bytes_with_nul()).unwrap();

    // Properly access static mut with explicit unsafe block
    let env_count =
        unsafe { (*std::ptr::addr_of_mut!(ENV_COUNT)).load(std::sync::atomic::Ordering::SeqCst) };
    for i in 0..env_count {
        let ptr = unsafe { ENV_PTRS[i].load(std::sync::atomic::Ordering::SeqCst) };
        if !ptr.is_null() {
            let s = unsafe { CStr::from_ptr(ptr as *const i8) };
            if s.to_bytes().starts_with(name.to_bytes())
                && s.to_bytes().get(name.to_bytes().len()) == Some(&b'=')
            {
                unsafe {
                    ENV_STORE[i][..buf.len()].copy_from_slice(&buf);
                    ENV_PTRS[i].store(ptr, std::sync::atomic::Ordering::SeqCst);
                    return;
                }
            }
        }
    }
    // Fix boundary check - array indices are 0-based and we're using env_count as an index
    if env_count >= MAX_ENV_COUNT {
        return;
    }
    unsafe {
        let ptr = &mut ENV_STORE[env_count];
        ptr[..buf.len()].copy_from_slice(&buf);
        ENV_PTRS[env_count].store(ptr.as_mut_ptr(), std::sync::atomic::Ordering::SeqCst);
        // Properly access static mut with explicit unsafe block
        (*std::ptr::addr_of_mut!(ENV_COUNT)).fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

fn printenv_r() {
    unsafe {
        let n = (*std::ptr::addr_of_mut!(ENV_COUNT)).load(std::sync::atomic::Ordering::SeqCst);
        for i in 0..n {
            let ptr = ENV_PTRS[i].load(std::sync::atomic::Ordering::SeqCst);
            let s = CStr::from_ptr(ptr as *const i8);
            println!("{:?}", s);
        }
    }
}
