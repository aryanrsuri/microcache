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
    a: f64,
}

impl Map {
    pub fn new() -> Self {
        return Map {
            array: vec![None; 3],
            version: [0; 10],
            a: 0.,
        };
    }

    fn hash(&mut self, key: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for &c in key {
            h = (h ^ c as u64).wrapping_mul(0x100000001b3);
        }
        return h;
    }

    /// WIP
    pub fn get(&mut self, key: &[u8]) -> Option<Block> {
        let i = self.hash(key) as usize % self.array.len();

        while self.array[i].is_some() {}

        None
    }

    pub fn set(&mut self, key: &[u8], value: &[u8]) {
        if self.a > 0.5 {
            self.array.resize(2 * self.array.len(), None);
        }
        let h = self.hash(key);
        let k = h as usize % self.array.len();
        let mut i = k;

        while self.array[i].is_some() {
            // FIXME: Is this really the most perf way to do this?
            if self.array[i].to_owned().unwrap().hash == h {
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
        self.a = self.array.iter().filter(|x| x.is_some()).count() as f64 / self.array.len() as f64;
    }

    fn increment(&mut self) -> [u8; 10] {
        let i = self.version[..8]
            .iter()
            .rposition(|&b| b != u8::MAX)
            .expect("version exauhsted");
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
