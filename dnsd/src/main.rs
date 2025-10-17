use dnsd::server;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .init();
    server::serve()?;
    Ok(())
}

#[test]
fn test() {
    let output = std::process::Command::new("dig")
        .args(["@127.0.0.1", "-p", "8000", "example.com"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("{}", stdout);
    eprintln!("{}", stderr);
}
