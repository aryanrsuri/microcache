//! Implements hashtable for use in cache
//! LICENCE: AGPL; Aryan Suri 2026

use core::str;

use crate::hashtable::ERROR::KEY_NOT_FOUND;

#[derive(Hash, Clone)]
pub struct Block {
    hash: u64,
    version: [u8; 10],
    key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Map {
    array: Vec<Option<Block>>,
    version: [u8; 10],
    fill: u64,
}

pub enum ERROR {
    KEY_NOT_FOUND,
    NOT_VALID_INCR,
    NOT_VALID_DECR,
}

impl Map {
    pub fn new() -> Self {
        return Map {
            array: vec![None; 4],
            version: [0; 10],
            fill: 0,
        };
    }

    pub fn get(&mut self, key: &[u8]) -> Option<&Block> {
        let h = self.hash(key);
        let k = self.index(h);
        let mut i = k;
        while let Some(block) = &self.array[i] {
            if block.hash == h && block.key == key {
                return Some(block);
            }
            i = (i + 1) % self.array.len();
            if i == k {
                break;
            }
        }

        None
    }

    // TODO: Only set if new key
    pub fn set(&mut self, key: &[u8], value: &[u8]) {
        let h = self.hash(key);
        let mut i = self.index(h);
        let version = self.increment();

        // FIXME: Can I simplify this in accordance w/ `delete` and `incr`
        while let Some(block) = &mut self.array[i] {
            if block.hash == h && block.key == key {
                block.value.clear();
                block.value.extend_from_slice(value);
                block.version = version;
                return;
            }
            i = (i + 1) % self.array.len();
        }

        // Resize and re-index before new key-set.
        if (self.fill as f64 / self.array.len() as f64) > 0.650 {
            self.resize(self.array.len() * 2);
            i = self.index(h);
            while self.array[i].is_some() {
                i = (i + 1) % self.array.len();
            }
        }

        self.array[i] = Some(Block {
            hash: h,
            version: version,
            key: key.to_vec(),
            value: value.to_vec(),
        });
        self.fill += 1;
    }

    /// Atomically increment a value by 1.
    /// Panics if the value is not parsable as an integer.
    pub fn incr(&mut self, key: &[u8]) {
        let h = self.hash(key);
        let mut i = self.index(h);
        let k = i;
        let version = self.increment();

        loop {
            // If key does not exist, initialise it to 1
            let Some(block) = &mut self.array[i] else {
                self.set(key, b"1");
                break;
            };

            if block.hash == h && block.key == key {
                block.value = (str::from_utf8(block.value.as_slice())
                    .expect("[ERR] Value is not an integer.")
                    .parse::<i64>()
                    .expect("[ERR] Could not parse integer.")
                    + 1)
                .to_string()
                .into_bytes();
                block.version = version;
                break;
            }

            i = (i + 1) % self.array.len();
            if i == k {
                break;
            }
        }
    }

    pub fn decr(&mut self, key: &[u8]) -> Option<ERROR> {
        let h = self.hash(key);
        let mut i = self.index(h);
        let k = i;
        let version = self.increment();

        loop {
            let Some(block) = &mut self.array[i] else {
                return Some(KEY_NOT_FOUND);
            };

            if block.hash == h && block.key == key {
                block.value = (str::from_utf8(block.value.as_slice())
                    .expect("[ERR] Value is not an integer.")
                    .parse::<i64>()
                    .expect("[ERR] Could not parse integer.")
                    - 1)
                .to_string()
                .into_bytes();
                block.version = version;
                break;
            }

            i = (i + 1) % self.array.len();
            if i == k {
                break;
            }
        }
        None
    }

    // Need to re-understand this implementation
    pub fn delete(&mut self, key: &[u8]) -> bool {
        let h = self.hash(key);
        let mut i = self.index(h);
        let k = i;

        loop {
            let Some(block) = &self.array[i] else {
                return false;
            };

            if block.hash == h && block.key == key {
                break;
            }

            i = (i + 1) % self.array.len();
            if i == k {
                return false;
            }
        }

        self.array[i] = None;
        self.fill -= 1;

        let mut start = i;
        i = (i + 1) % self.array.len();

        while let Some(block) = &self.array[i] {
            let end = self.index(block.hash);
            let dist_to_start = (start + self.array.len() - end) % self.array.len();
            let dist_to_slot = (i + self.array.len() - end) % self.array.len();
            // Move blocks where path crosses start
            if dist_to_start < dist_to_slot {
                self.array[start] = self.array[i].take();
                start = i;
            }
            i = (i + 1) % self.array.len();
        }

        true
    }

    fn resize(&mut self, scalar: usize) {
        let first = std::mem::take(&mut self.array);
        self.array.resize(scalar, None);
        for block in first.into_iter().flatten() {
            let mut i = self.index(block.hash);
            while self.array[i].is_some() {
                i = (i + 1) % self.array.len();
            }
            self.array[i] = Some(block)
        }
    }

    fn index(&self, hash: u64) -> usize {
        hash as usize % self.array.len()
    }

    fn hash(&self, key: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for &c in key {
            h = (h ^ c as u64).wrapping_mul(0x100000001b3);
        }
        return h;
    }

    fn increment(&mut self) -> [u8; 10] {
        let i = self.version[..8]
            .iter()
            .rposition(|&b| b != u8::MAX)
            .expect("version exhausted");
        self.version[i] += 1;
        self.version[i + 1..8].fill(0);
        self.version
    }
}

impl std::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block")
            .field("hash", &format!("0x{:016x}", self.hash))
            .field("version", &hex(&self.version))
            .field("key", &hex(&self.key))
            .field("value", &hex(&self.value))
            .finish()
    }
}

pub fn hex(bytes: &[u8]) -> String {
    let digits: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("0x{digits}")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Map;

    #[test]
    fn can_get() {
        let mut map = Map::new();
        map.set(b"Bob", b"100");
        assert_eq!(map.get(b"Bob").unwrap().value, b"100");
        assert!(map.get(b"None").is_none());
    }

    #[test]
    fn can_incr() {
        let mut map = Map::new();
        map.set(b"Bob", b"2");
        assert_eq!(map.get(b"Bob").unwrap().value, b"2");
        map.incr(b"Bob");
        assert_eq!(map.get(b"Bob").unwrap().value, b"3");
    }

    #[test]
    fn can_decr() {
        let mut map = Map::new();
        map.set(b"Bob", b"2");
        assert_eq!(map.get(b"Bob").unwrap().value, b"2");
        map.decr(b"Bob");
        assert_eq!(map.get(b"Bob").unwrap().value, b"1");
    }

    #[test]
    fn can_resize() {
        let mut map = Map::new();
        map.set(b"a", b"old");
        map.set(b"b", b"other");
        map.set(b"a", b"new");

        assert_eq!(map.get(b"a").unwrap().value, b"new");
        assert_eq!(map.get(b"b").unwrap().value, b"other");

        let occupied = map.array.iter().flatten().count();
        assert_eq!(occupied, 2);
    }

    #[test]
    fn can_delete() {
        let mut map = Map::new();
        map.set(b"DEL", b"old");
        map.set(b"KEEP", b"other");
        map.set(b"DEL", b"new");

        assert_eq!(map.get(b"DEL").unwrap().value, b"new");
        assert_eq!(map.get(b"KEEP").unwrap().value, b"other");

        map.delete(b"DEL");
        assert!(map.get(b"DEL").is_none());
        assert!(map.get(b"KEEP").is_some());
        let occupied = map.array.iter().flatten().count();
        assert_eq!(occupied, 1);
    }

    #[test]
    fn can_perf() {
        use std::hint::black_box;
        use std::time::{Duration, Instant};

        const ITER: u32 = 5;
        let scales = [1_u64, 10, 100, 1_000, 10_000, 100_000, 1_000_000];

        println!("\nInsert benchmark ({ITER} runs per size; both maps start empty)");
        println!(
            "{:>10} {:>14} {:>14} {:>14} {:>14} {:>10}",
            "Entries", "Map total", "Std total", "Map ns/set", "Std ns/set", "Std/Map",
        );

        for scale in scales {
            let bench_map = || {
                let mut map = Map::new();
                let start = Instant::now();
                for i in 0..scale {
                    let bytes = i.to_le_bytes();
                    map.set(&bytes, &bytes);
                }
                black_box(&map);
                let elapsed = start.elapsed();

                // Check correctness outside the timed region.
                assert_eq!(map.fill, scale);
                for i in 0..scale {
                    let bytes = i.to_le_bytes();
                    assert_eq!(map.get(&bytes).unwrap().value.as_slice(), &bytes);
                }
                elapsed
            };
            let bench_std = || {
                let mut map = HashMap::<Vec<u8>, Vec<u8>>::new();
                let start = Instant::now();
                for i in 0..scale {
                    let bytes = i.to_le_bytes();
                    map.insert(bytes.to_vec(), bytes.to_vec());
                }
                black_box(&map);
                let elapsed = start.elapsed();

                assert_eq!(map.len(), scale as usize);
                for i in 0..scale {
                    let bytes = i.to_le_bytes();
                    assert_eq!(map.get(bytes.as_slice()).unwrap().as_slice(), &bytes);
                }
                elapsed
            };

            let mut map_total = Duration::ZERO;
            let mut std_total = Duration::ZERO;
            for iteration in 0..ITER {
                // Alternate order to reduce bias from always running one map first.
                let (map_elapsed, std_elapsed) = if iteration % 2 == 0 {
                    (bench_map(), bench_std())
                } else {
                    let std_elapsed = bench_std();
                    (bench_map(), std_elapsed)
                };
                map_total += map_elapsed;
                std_total += std_elapsed;
            }

            let map_mean = map_total / ITER;
            let std_mean = std_total / ITER;
            let operations = f64::from(ITER) * scale as f64;
            let map_ns = map_total.as_secs_f64() * 1e9 / operations;
            let std_ns = std_total.as_secs_f64() * 1e9 / operations;
            println!(
                "{scale:>10} {map_mean:>14.3?} {std_mean:>14.3?} {map_ns:>14.2} {std_ns:>14.2} {:>9.2}x",
                std_ns / map_ns,
            );
        }
    }
}
