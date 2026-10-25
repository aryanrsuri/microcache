use std::io::{self, Write};

use crate::hashtable::Map;

pub fn repl(mut m: Map) {
    loop {
        print!("repl$ ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin
            .read_line(&mut buffer)
            .expect("[ERR] can't read input.");
        let parts: Vec<&str> = buffer.split_whitespace().collect();

        match parts.as_slice() {
            [] => {}
            ["SET", key, value] => m.set(key.as_bytes(), value.as_bytes()),
            ["GET", key] => match m.get(key.as_bytes()) {
                Some(block) => {
                    let value =
                        String::from_utf8(block.value.clone()).expect("Could not parse value.");
                    println!("{:?}", value);
                }
                None => println!("Key not found."),
            },
            ["DEL", key] => {
                if m.delete(key.as_bytes()) {
                    println!("OK");
                } else {
                    println!("Key not found")
                }
            }
            ["INCR", key] => {
                m.incr(key.as_bytes());
                let value =
                    String::from_utf8(m.get(key.as_bytes()).expect("Key not found.").value.clone())
                        .expect("Could not parse value.");
                println!("{:?}", value);
            }
            ["DECR", key] => {
                m.decr(key.as_bytes());
                let value =
                    String::from_utf8(m.get(key.as_bytes()).expect("Key not found.").value.clone())
                        .expect("Could not parse value.");
                println!("{:?}", value);
            }
            [".exit"] => break,
            [".help"] => println!("SET key value; GET key; INCR key; DECR key; .exit; .help"),
            _ => println!("repl$ invalid command, see .help for commands"),
        }
    }
}
