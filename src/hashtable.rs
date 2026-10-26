//! Implements hashtable for use in cache
//! LICENCE: AGPL; Aryan Suri 2026

use core::str;
use std::time::{Duration, Instant};

use crate::hashtable::CacheError::KeyNotFound;

const MAX_KEY_SIZE: usize = 256;
const MAX_VAL_SIZE: usize = 1_048_576;

#[derive(Debug)]
pub enum CacheError {
    KeyNotFound,
    KeyLenExceeded,
    ValueLenExceeded,
    InvalidValue,
}

#[derive(Hash, Clone)]
pub struct Block {
    hash: u64,
    version: [u8; 10],
    key: Vec<u8>,
    pub value: Vec<u8>,
    exptime: Option<std::time::Instant>,
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
                if let Some(ttl) = block.exptime
                    && Instant::now() >= ttl
                {
                    return None;
                }
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
    pub fn set(
        &mut self,
        key: &[u8],
        value: &[u8],
        exptime: Option<Duration>,
    ) -> Result<(), CacheError> {
        if key.len() > MAX_KEY_SIZE {
            return Err(CacheError::KeyLenExceeded);
        }
        if value.len() > MAX_VAL_SIZE {
            return Err(CacheError::ValueLenExceeded);
        }
        let h = self.hash(key);
        let mut i = self.index(h);
        let ttl = exptime.map(|ttl| Instant::now() + ttl);
        let version = self.increment();

        // FIXME: Can I simplify this in accordance w/ `delete` and `incr`
        while let Some(block) = &mut self.array[i] {
            if block.hash == h && block.key == key {
                block.value.clear();
                block.value.extend_from_slice(value);
                block.version = version;
                block.exptime = ttl;
                return Ok(());
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
            exptime: ttl,
        });
        self.fill += 1;
        Ok(())
    }

    pub fn incr(&mut self, key: &[u8]) -> Result<i64, CacheError> {
        let h = self.hash(key);
        let mut i = self.index(h);
        let k = i;

        loop {
            // If key does not exist, initialise it to 1
            let Some(block) = &mut self.array[i] else {
                self.set(key, b"1", None)?;
                return Ok(1);
            };

            if block.hash == h && block.key == key {
                let new = str::from_utf8(block.value.as_slice())
                    .map_err(|_| CacheError::InvalidValue)?
                    .parse::<i64>()
                    .map_err(|_| CacheError::InvalidValue)?
                    .checked_add(1)
                    .ok_or(CacheError::InvalidValue)?;
                let version = self.increment();
                let block = self.array[i].as_mut().unwrap();
                block.value = new.to_string().into_bytes();
                block.version = version;
                return Ok(new);
            }

            i = (i + 1) % self.array.len();
            if i == k {
                break;
            }
        }
        return Err(KeyNotFound);
    }

    pub fn decr(&mut self, key: &[u8]) -> Result<i64, CacheError> {
        let h = self.hash(key);
        let mut i = self.index(h);
        let k = i;

        loop {
            // Not sure if this is a good default behaviour
            let Some(block) = &mut self.array[i] else {
                self.set(key, b"0", None)?;
                return Ok(0);
            };

            if block.hash == h && block.key == key {
                let new = str::from_utf8(block.value.as_slice())
                    .map_err(|_| CacheError::InvalidValue)?
                    .parse::<i64>()
                    .map_err(|_| CacheError::InvalidValue)?
                    .checked_sub(1)
                    .ok_or(CacheError::InvalidValue)?;
                let version = self.increment();
                let block = self.array[i].as_mut().unwrap();
                block.value = new.to_string().into_bytes();
                block.version = version;
                return Ok(new);
            }

            i = (i + 1) % self.array.len();
            if i == k {
                break;
            }
        }

        return Err(KeyNotFound);
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
        let i = self.version[..10]
            .iter()
            .rposition(|&b| b != u8::MAX)
            .expect("version exhausted");
        self.version[i] += 1;
        self.version[i + 1..10].fill(0);
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
    use std::time::{Duration, Instant};

    use super::Map;

    #[test]
    fn can_get() {
        let mut map = Map::new();
        map.set(b"Bob", b"100", None).unwrap();
        assert_eq!(map.get(b"Bob").unwrap().value, b"100");
        assert!(map.get(b"None").is_none());
    }

    #[test]
    fn can_incr() {
        let mut map = Map::new();
        map.set(b"Bob", b"2", None).unwrap();
        assert_eq!(map.get(b"Bob").unwrap().value, b"2");
        map.incr(b"Bob").unwrap();
        assert_eq!(map.get(b"Bob").unwrap().value, b"3");
    }

    #[test]
    fn can_decr() {
        let mut map = Map::new();
        map.set(b"Bob", b"2", None).unwrap();
        assert_eq!(map.get(b"Bob").unwrap().value, b"2");
        map.decr(b"Bob").unwrap();
        assert_eq!(map.get(b"Bob").unwrap().value, b"1");
    }

    #[test]
    fn can_expire() {
        let mut map = Map::new();
        map.set(b"Bob", b"1", Some(Duration::from_secs(1))).unwrap();
        let end = Instant::now() + Duration::from_secs(1);
        while Instant::now() < end {}
        assert!(map.get(b"Bob").is_none());
    }

    #[test]
    fn can_persist() {
        let mut map = Map::new();
        map.set(b"Bob", b"1", Some(Duration::from_secs(100)))
            .unwrap();
        let end = Instant::now() + Duration::from_secs(1);
        while Instant::now() < end {}
        assert_eq!(map.get(b"Bob").unwrap().value, b"1");
    }
    #[test]
    fn can_resize() {
        let mut map = Map::new();
        map.set(b"a", b"old", None).unwrap();
        map.set(b"b", b"other", None).unwrap();
        map.set(b"a", b"new", None).unwrap();
        map.set(b"c", b"new", None).unwrap();
        map.set(b"d", b"new", None).unwrap();
        map.set(b"e", b"new", None).unwrap();
        map.set(b"f", b"new", None).unwrap();
        map.set(b"g", b"new", None).unwrap();

        assert_eq!(map.get(b"a").unwrap().value, b"new");
        assert_eq!(map.get(b"b").unwrap().value, b"other");

        let occupied = map.array.iter().flatten().count();
        assert_eq!(occupied, 7);
    }

    #[test]
    fn can_delete() {
        let mut map = Map::new();
        map.set(b"DEL", b"old", None).unwrap();
        map.set(b"KEEP", b"other", None).unwrap();
        map.set(b"DEL", b"new", None).unwrap();

        assert_eq!(map.get(b"DEL").unwrap().value, b"new");
        assert_eq!(map.get(b"KEEP").unwrap().value, b"other");

        map.delete(b"DEL");
        assert!(map.get(b"DEL").is_none());
        assert!(map.get(b"KEEP").is_some());
        let occupied = map.array.iter().flatten().count();
        assert_eq!(occupied, 1);
    }
}
