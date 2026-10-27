use std::{
    io::{self, Write},
    time::Duration,
};

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
            ["SET", key, value] => match m.set(key.as_bytes(), value.as_bytes(), None) {
                Ok(_) => println!("OK"),
                Err(err) => println!("{:?}", err),
            },
            ["SET", key, value, exptime] => {
                match m.set(
                    key.as_bytes(),
                    value.as_bytes(),
                    Some(Duration::from_secs(exptime.parse::<u64>().unwrap())),
                ) {
                    Ok(_) => println!("OK"),
                    Err(err) => println!("{:?}", err),
                }
            }
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
            ["INCR", key] => match m.incr(key.as_bytes()) {
                Ok(n) => println!("{}", n),
                Err(err) => println!("{:?}", err),
            },
            ["DECR", key] => match m.decr(key.as_bytes()) {
                Ok(n) => println!("{}", n),
                Err(err) => println!("{:?}", err),
            },
            [] => {}
            [".exit"] => break,
            [".help"] => {
                println!("SET key value; GET key; GET_BLOCK key; INCR key; DECR key; .exit; .help")
            }
            _ => println!("repl$ invalid command, see .help for commands"),
        }
    }
}
