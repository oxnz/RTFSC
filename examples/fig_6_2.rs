use rtfsc::passwd::PasswdReader;

fn main() {
    let iter = PasswdReader::default();
    for item in iter {
        println!("{item:?}");
    }
}
