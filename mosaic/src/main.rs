use std::{
    ffi::{CString, c_void},
    path::PathBuf,
    process::ExitCode,
    str::FromStr,
};

use futures::{StreamExt, TryFutureExt, stream};
use reqwest::Url;

#[tokio::main]
async fn main() -> Result<(), ExitCode> {
    let chunk_size = 4 * 1024 * 1024;
    let argv = std::env::args().collect::<Vec<_>>();
    let (uri, path) = match argv.len() {
        2 => {
            let uri = Url::try_from(argv[1].as_str()).expect("invalid uri");
            let path = PathBuf::try_from(uri.path()).expect("invalid path");
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .expect("missing filename");
            (
                uri.to_string(),
                CString::from_str(name).expect("invalid path"),
            )
        }
        3 => (
            argv[1].to_string(),
            CString::from_str(&argv[2]).expect("invalid path"),
        ),
        _ => {
            eprintln!("usage: {} <uri> [pathname]", argv[0]);
            return Err(ExitCode::FAILURE);
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let client = reqwest::Client::new();
    let resp = client.head(&uri).send().await.unwrap();
    let headers = resp.headers();
    headers
        .get(reqwest::header::ACCEPT_RANGES)
        .expect("do not support range");
    let size: usize = resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .expect("missing content-length")
        .to_str()
        .expect("invalid value")
        .parse()
        .unwrap();
    tracing::info!("content-length: {size}");
    let fd = unsafe {
        let fd = libc::open(
            path.as_ptr(),
            libc::O_CREAT | libc::O_WRONLY,
            libc::S_IRUSR | libc::S_IWUSR | libc::S_IRGRP,
        );
        if fd < 0 {
            tracing::error!("failed to open: {:?}", std::io::Error::last_os_error());
            return Err(ExitCode::FAILURE);
        }
        if libc::fallocate(fd, 0, 0, size as i64) < 0 {
            tracing::error!("fallocate: {:?}", std::io::Error::last_os_error());
            return Err(ExitCode::FAILURE);
        }
        if libc::fsetxattr(
            fd,
            "mosaic.uri\0".as_ptr(),
            path.as_ptr() as *const c_void,
            path.count_bytes(),
            0,
        ) < 0
        {
            tracing::warn!("fsetxattr: {:?}", std::io::Error::last_os_error());
        }
        if libc::fsetxattr(
            fd,
            "mosaic.chunk_size\0".as_ptr(),
            std::ptr::addr_of!(chunk_size) as *const c_void,
            size_of::<i64>(),
            0,
        ) < 0
        {
            tracing::warn!("fsetxattr: {:?}", std::io::Error::last_os_error());
        }
        fd
    };
    stream::iter((0..size).step_by(chunk_size).map(|offset| {
        (
            offset,
            if offset + chunk_size < size {
                chunk_size
            } else {
                size % chunk_size
            },
        )
    }))
    .map(|(offset, len)| (offset, fetch(&client, &uri, offset, len)))
    .for_each_concurrent(32, |(offset, fut)| async move {
        match fut.await {
            Ok(content) => {
                if unsafe {
                    libc::pwrite(
                        fd,
                        content.as_ptr() as *const c_void,
                        content.len(),
                        offset as i64,
                    )
                } != content.len() as isize
                {
                    tracing::error!("write error");
                }
                tracing::info!("done for offset: {offset}, len: {}", content.len());
            }
            Err(e) => {
                tracing::error!("failed to fetch chunk with offset: {offset}, error: {e:?}");
            }
        }
    })
    .await;

    tracing::info!("Hello, world!");

    Ok(())
}

async fn fetch(
    client: &reqwest::Client,
    uri: &str,
    start: usize,
    len: usize,
) -> reqwest::Result<bytes::Bytes> {
    loop {
        match client
            .get(uri)
            .header(
                reqwest::header::RANGE,
                format!("bytes={}-{}", start, start + len),
            )
            .send()
            .and_then(|resp| async move { resp.bytes().await })
            .await
        {
            Ok(content) => return Ok(content),
            Err(e) => {
                tracing::warn!("retry for error: {e:?}");
                continue;
            }
        }
    }
}
