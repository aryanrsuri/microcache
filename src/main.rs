use crate::{benchmark::compare, repl::repl};
use std::env;

mod benchmark;
mod hashtable;
mod repl;

fn main() {
    let m = hashtable::Map::new();
    let args = env::args().collect::<Vec<String>>();
    if args.len() > 1 && args[1] == "--benchmark" {
        compare();
    } else {
        repl(m);
    }
}
