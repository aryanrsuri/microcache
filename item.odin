package main

import "core:time"
MAX_KEY_LEN: int : 255
MAX_VAL_LEN: int : 1_000_000

item :: struct {
	prev:        int,
	next:        int,
	accessed_at: i64,
	expires_at:  i64,
	key_len:     int,
	val_len:     int,
	bytes:       [dynamic]u8,
}

// Create a new item allocation
// key: []u8
// value: []u8
// prev, next: int
// expires_at: i64
item_create :: proc(key, value: []u8, prev, next: int, expires_at: i64) -> (item, int) {
	key_len, val_len := len(key), len(value)
	if len(key) > MAX_KEY_LEN {
		panic("key len too large")
	}
	if len(value) > MAX_VAL_LEN {
		panic("value len too large")
	}

	bytes := make([dynamic]u8, key_len + val_len)
	for i := 0; i < key_len; i += 1 {
		inject_at(&bytes, i, key[i])
	}
	for i := key_len; i < (key_len + val_len); i += 1 {
		inject_at(&bytes, i, value[i - val_len])
	}

	return {
			prev = prev,
			next = next,
			accessed_at = time.now()._nsec,
			expires_at = expires_at,
			key_len = key_len,
			val_len = val_len,
			bytes = bytes,
		},
		key_len + val_len

}

// Write a new value to the item
// item: ^item
// value: []u8
// expires_at: i64
item_update_value :: proc(item: ^item, value: []u8, expires_at: i64) -> int {
	used := item.val_len - len(value)
	if len(value) > MAX_VAL_LEN {
		panic("value len too large")
	}
	item.val_len = len(value)
	resize_dynamic_array(&item.bytes, item.key_len + item.val_len)
	for i := item.key_len; i < item.val_len; i += 1 {
		item.bytes[i] = value[i]
	}
	item.expires_at = expires_at
	item.accessed_at = time.now()._nsec

	return used
}
