#[derive(Hash, Clone)]
pub struct Block {
    hash: u64,
    version: [u8; 10],
    key: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Map {
    array: Vec<Option<Block>>,
    version: [u8; 10],
    fill: u64,
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

    pub fn set(&mut self, key: &[u8], value: &[u8]) {
        let h = self.hash(key);
        let mut i = self.index(h);
        let version = self.increment();

        while let Some(block) = &mut self.array[i] {
            if block.hash == h && block.key == key {
                block.version = version;
                block.value.clear();
                block.value.extend_from_slice(value);
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

    // Need to re-understand this implementation
    pub fn delete(&mut self, key: &[u8]) -> bool {
        let h = self.hash(key);
        let mut i = self.index(h);

        loop {
            let Some(block) = &self.array[i] else {
                return false;
            };

            if block.hash == h && block.key == key {
                break;
            }

            i = (i + 1) % self.array.len()
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
        const ITER: usize = 1;
        let scales = [1_u64, 10, 100, 1_000, 10_000, 100_000, 1_000_000];
        let mut timings = vec![[std::time::Duration::ZERO; ITER]; scales.len()];

        println!("");
        for iteration in 0..ITER {
            for (scale_index, scale) in scales.into_iter().enumerate() {
                let mut map = Map::new();
                let start = std::time::Instant::now();

                for i in 0..scale {
                    let bytes = i.to_le_bytes();
                    map.set(&bytes, &bytes);
                }

                std::hint::black_box(&map);
                let elapsed = start.elapsed();
                timings[scale_index][iteration] = elapsed;

                println!(
                    "run={} scale={scale:>7} total={elapsed:?} avg/set={:?}",
                    iteration + 1,
                    elapsed / scale as u32,
                );
            }
        }

        if ITER > 1 {
            println!("\nSummary ({} runs per scale)", ITER);
            println!(
                "{:>10} {:>14} {:>14} {:>14} {:>14}",
                "Scale", "Mean total", "Min total", "Max total", "Mean/set (ns)",
            );
            for (scale, samples) in scales.into_iter().zip(timings) {
                let total = samples.iter().copied().sum::<std::time::Duration>();
                let mean = total / samples.len() as u32;
                let per_set = total.as_secs_f64() * 1e9 / samples.len() as f64 / scale as f64;
                let min = samples.iter().min().unwrap();
                let max = samples.iter().max().unwrap();
                println!("{scale:>10} {mean:>14.3?} {min:>14.3?} {max:>14.3?} {per_set:>14.3}",);
            }
        }
    }
}
