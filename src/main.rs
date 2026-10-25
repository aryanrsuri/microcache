use crate::{benchmark::compare, repl::repl};

mod benchmark;
mod hashtable;
mod repl;

fn main() {
    let m = hashtable::Map::new();
    if false {
        compare();
    }
    repl(m);
}
