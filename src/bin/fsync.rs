use std::os::fd::AsRawFd;
use std::os::unix::io::RawFd;
use std::time::{Duration, Instant};
use std::{io, sync::Arc};

use libc::{F_GLOBAL_NOCACHE, F_NOCACHE, fcntl};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Barrier;

const FILE_PATH: &str = "./block.bin";
const FILE_SIZE: u64 = 8 * 1024 * 1024 * 1024; // 8 GiB
const BLOCK_SIZE: usize = 1 * 1024 * 1024; // 1 MiB
const NUM_TASKS: usize = 16;
const TEST_DURATION: Duration = Duration::from_secs(60 * 1000);
const WRITE_MODE: bool = true; // true = write test, false = read test

fn disable_cache(fd: RawFd) {
    unsafe {
        // macOS-specific
        fcntl(fd, F_NOCACHE, 1);
        fcntl(fd, F_GLOBAL_NOCACHE, 1);
        // posix_fadvise(fd, 0, 0, POSIX_FADV_RANDOM);
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Preallocate test file if in write mode
    if WRITE_MODE {
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(FILE_PATH)?;
        f.set_len(FILE_SIZE)?;
    }

    let barrier = Arc::new(Barrier::new(NUM_TASKS));
    let mut handles = tokio::task::JoinSet::new();

    for tid in 0..NUM_TASKS {
        let b = barrier.clone();
        handles.spawn(async move {
            let f = OpenOptions::new()
                .read(true)
                .write(WRITE_MODE)
                .open(FILE_PATH)
                .await
                .expect("open file");

            // disable OS caching for this fd
            disable_cache(f.as_raw_fd());

            let region_size = FILE_SIZE / NUM_TASKS as u64;
            let mut buf = vec![0u8; BLOCK_SIZE];
            if WRITE_MODE {
                for b in &mut buf {
                    *b = rand::random::<u8>();
                }
            }

            b.wait().await;
            let start = Instant::now();
            let mut bytes_done = 0u64;
            let mut file = f;

            while start.elapsed() < TEST_DURATION {
                let offset = (rand::random::<u64>() % region_size) + (tid as u64 * region_size);
                file.seek(std::io::SeekFrom::Start(offset)).await.unwrap();

                if WRITE_MODE {
                    file.write_all(&buf).await.unwrap();
                } else {
                    file.read_exact(&mut buf).await.unwrap();
                }

                bytes_done += BLOCK_SIZE as u64;
            }

            let secs = start.elapsed().as_secs_f64();
            println!("Task {tid}: {:.2} MB/s", bytes_done as f64 / secs / 1e6);
            bytes_done
        });
    }

    let mut total = 0u64;
    handles.join_all().await.iter().for_each(|sz| {
        total += sz;
    });

    println!(
        "\nTotal throughput: {:.2} MB/s",
        total as f64 / TEST_DURATION.as_secs_f64() / 1e6
    );

    Ok(())
}
