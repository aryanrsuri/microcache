#[derive(Hash, Clone)]
pub struct Block {
    hash: u64,
    version: [u8; 10],
    key: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Map {
    pub array: Vec<Option<Block>>,
    version: [u8; 10],
    fill: u64,
}

impl Map {
    pub fn new() -> Self {
        return Map {
            array: vec![None; 3],
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

    pub fn set(&mut self, key: &[u8], value: &[u8]) {
        if (self.fill as f64 / self.array.len() as f64) > 0.5 {
            self.resize();
        }
        let h = self.hash(key);
        let k = self.index(h);
        let mut overwrite = false;
        let mut i = k;

        while let Some(block) = &self.array[i] {
            if block.hash == h && block.key == key {
                overwrite = true;
                break;
            }
            i = (i + 1) % self.array.len();
        }
        let b = Block {
            hash: h,
            version: self.increment(),
            key: key.to_vec(),
            value: value.to_vec(),
        };
        self.array[i] = Some(b);
        if !overwrite {
            self.fill += 1;
        }
    }

    fn resize(&mut self) {
        let first = std::mem::take(&mut self.array);
        self.array.resize(first.len() * 2, None);
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
        let hex = |bytes: &[u8]| {
            let digits: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            format!("0x{digits}")
        };

        f.debug_struct("Block")
            .field("hash", &format!("0x{:016x}", self.hash))
            .field("version", &hex(&self.version))
            .field("key", &hex(&self.key))
            .field("value", &hex(&self.value))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::Map;

    #[test]
    fn can_get() {
        let mut map = Map::new();
        map.set(b"Bob", b"100");
        assert_eq!(map.get(b"Bob").unwrap().value, b"100");
        assert!(map.get(b"None").is_none());
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
    #[ignore = "run explicitly in release mode"]
    fn can_perf() {
        let scales = [1_u64, 10, 100, 1_000, 10_000, 100_000];

        for scale in scales {
            let mut map = Map::new();
            let start = std::time::Instant::now();

            for i in 0..scale {
                let bytes = i.to_le_bytes();
                map.set(&bytes, &bytes);
            }

            std::hint::black_box(&map);
            let elapsed = start.elapsed();

            println!(
                "scale={scale:>6} total={elapsed:?} avg/set={:?}",
                elapsed / scale as u32,
            );
        }
    }
}
