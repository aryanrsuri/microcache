mod hashtable;

fn main() {
    let mut m = hashtable::Map::new();
    m.set("Bob".as_bytes(), b"100");
    m.set("Foo".as_bytes(), b"10");
    m.set("Bar".as_bytes(), b"200");
    m.set("Baz".as_bytes(), b"140");
    m.set("Bob".as_bytes(), b"400");

    println!("{:?}", m.get("Baz".as_bytes()).unwrap());
    println!("{:?}", m.get("Miss".as_bytes()));
    m.incr("Bob".as_bytes());
    m.delete("Bar".as_bytes());
    println!("{:?}", m.get("Bar".as_bytes()));
}
