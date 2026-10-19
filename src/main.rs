mod cache;

fn main() {
    let mut m = cache::Map::new();
    m.set("Bob".as_bytes(), &100_usize.to_le_bytes());
    m.set("Foo".as_bytes(), &10_usize.to_le_bytes());
    m.set("Bar".as_bytes(), &200_usize.to_le_bytes());
    m.set("Baz".as_bytes(), &140_usize.to_le_bytes());
    m.set("Bob".as_bytes(), &400_usize.to_le_bytes());
    println!("{:#?}", m.array);

    println!("{:#?}", m.get("Baz".as_bytes()));
}
