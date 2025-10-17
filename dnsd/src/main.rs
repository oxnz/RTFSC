use dnsd::server;

fn main() -> std::io::Result<()> {
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
