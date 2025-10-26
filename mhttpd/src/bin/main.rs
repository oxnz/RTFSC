use mhttpd::server;
use tracing::Level;

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    tracing::debug!("Hello, world!");
    server::blocking::serve()?;
    tracing::debug!("bye");
    Ok(())
}

#[test]
fn test_get() {
    let output = std::process::Command::new("curl")
        .args(["-v", "--silent", "127.0.0.1:8000"])
        .output()
        .unwrap();
    let stdout = output.stdout;
    let stderr = output.stderr;
    println!("stdout:\n{:?}", HexRepr(&stdout));
    eprintln!("stderr:\n{:?}", HexRepr(&stderr));
}

#[test]
fn test_form() {
    let output = std::process::Command::new("curl")
        .args([
            "-X",
            "POST",
            "-d",
            "comment=hello",
            "-v",
            "--silent",
            "127.0.0.1:8000",
        ])
        .output()
        .unwrap();
    let stdout = output.stdout;
    let stderr = output.stderr;
    println!("stdout:\n{:?}", HexRepr(&stdout));
    eprintln!("stderr:\n{:?}", HexRepr(&stderr));
}

struct HexRepr<'a>(&'a [u8]);

impl<'a> std::fmt::Debug for HexRepr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return f.write_str("<empty>");
        }
        let s = String::from_utf8_lossy(self.0);
        return f.write_str(&s);

        for chunk in self.0.chunks(16) {
            for byte in chunk {
                write!(f, "{:x}", byte)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
