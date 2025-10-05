use acorn::Database;

fn main() {
    println!("{:?}", std::env::current_dir());
    let mut db = Database::open("test");
    assert_eq!(db.iter().count(), 0);
    db.store("alpha", "data 1").unwrap();
    db.store("beta", "record 2").unwrap();
    db.store("gamma", "record 3").unwrap();
    assert_eq!(db.iter().count(), 3);

    println!("Hello, world!");
}
