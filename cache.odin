package main

MAX_KEY_LEN: uint : 255
MAX_VAL_LEN: uint : 1_000_000

// an `item` is a byte encoding object
// that is contained within the cache.
item :: struct {
	prev:        int,
	next:        int,
	accessed_at: u64,
	expires_at:  u64,
	key_len:     int,
	val_len:     int,
	bytes:       [dynamic]u8,
}

// a `cache` is a struct to instantiate
// a new cache.
cache :: struct {
	index:      map[string]int,
	items:      [dynamic]item,
	free:       [dynamic]int,
	newest:     int,
	oldest:     int,
	used_bytes: int,
	max_bytes:  int,
}
