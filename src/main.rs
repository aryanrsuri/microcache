use crate::repl::repl;

mod hashtable;
mod repl;

fn main() {
    let m = hashtable::Map::new();
    repl(m);
}
