use std::ffi::c_void;

use futures::{StreamExt, TryFutureExt, stream};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();
    let uri = "https://proof.ovh.net/files/100Mb.dat";
    let uri = "https://old-releases.ubuntu.com/releases/8.04.0/ubuntu-8.04.4-desktop-amd64.iso";
    let retry_policy = reqwest::retry::for_host(uri)
        .classify_fn(|req_rep| {
            tracing::info!("retryable: {:?}", req_rep);
            match req_rep.status() {
                Some(reqwest::StatusCode::REQUEST_TIMEOUT) => req_rep.retryable(),
                // _ => req_rep.success()
                _ => req_rep.retryable(),
            }
        })
        .no_budget();
    let client = reqwest::ClientBuilder::new()
        .retry(retry_policy)
        .build()
        .unwrap();
    let resp = client.head(uri).send().await.unwrap();
    let headers = resp.headers();
    headers
        .get(reqwest::header::ACCEPT_RANGES)
        .expect("do not support range");
    let len: usize = resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .expect("no content length")
        .to_str()
        .expect("invalid value")
        .parse()
        .unwrap();
    tracing::info!("content-length: {len}");
    let fd = unsafe {
        let fd = libc::open(
            "mosaic/f.tmp\0".as_ptr(),
            libc::O_CREAT | libc::O_WRONLY,
            libc::S_IRUSR | libc::S_IWUSR | libc::S_IRGRP,
        );
        if fd < 0 {
            tracing::error!("failed to open");
            return;
        }
        if libc::fallocate(fd, 0, 0, len as i64) < 0 {
            tracing::error!("fallocate");
            return;
        }
        fd
    };
    let chunk_size = 4 * 1024 * 1024;
    stream::iter(
        (0..len)
            .step_by(chunk_size)
            .map(|offset| (offset, (offset + chunk_size - 1).min(len - 1))),
    )
    .map(|(offset, end)| (offset, fetch(&client, uri, offset, end)))
    .for_each_concurrent(32, |(offset, fut)| async move {
        match fut.await {
            Ok(content) => {
                let n = unsafe {
                    libc::pwrite(
                        fd,
                        content.as_ptr() as *const c_void,
                        content.len(),
                        offset as i64,
                    )
                };
                if n < 0 {
                    tracing::error!("write error");
                }
                tracing::info!("done for offset: {offset}, len: {}", content.len());
            }
            Err(e) => {
                tracing::error!("download failed for offset: {offset}, error: {e:?}");
            }
        }
    })
    .await;
    // libc::fallocate(fd, mode, offset, len)
    tracing::info!("Hello, world!");
}

async fn fetch(
    client: &reqwest::Client,
    uri: &str,
    start: usize,
    end: usize,
) -> Result<Vec<u8>, reqwest::Error> {
    loop {
        match client
            .get(uri)
            .header(reqwest::header::RANGE, format!("bytes={}-{}", start, end))
            .send()
            .and_then(|resp| async move { resp.bytes().await })
            .await
            .map(|bytes| bytes.to_vec())
        {
            Ok(content) => return Ok(content),
            Err(e) => {
                tracing::warn!("retry for error: {e:?}");
                continue;
            }
        }
    }
}
