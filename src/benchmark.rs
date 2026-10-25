use std::collections::HashMap;

use crate::hashtable::Map;

pub fn compare() {
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
