package main

import "core:sync"


// MUST Not be copied after first use
cache :: struct {
	lock:       sync.RW_Mutex,
	index:      map[string]int,
	items:      [dynamic]item,
	free:       [dynamic]int,
	newest:     int,
	oldest:     int,
	used_bytes: uint,
	max_bytes:  uint,
}

cache_create :: proc(max_bytes: uint) -> cache {
	return {
		index = make(map[string]int),
		items = make([dynamic]item),
		free = make([dynamic]int),
		newest = -1,
		oldest = -1,
		used_bytes = 0,
		max_bytes = max_bytes,
	}
}

cache_free :: proc(c: ^cache) {
	for &item, _ in c.items {delete_dynamic_array(item.bytes)}
	delete_dynamic_array(c.items)
	delete_dynamic_array(c.free)
	delete_map(c.index)
}

cache_set :: proc(c: ^cache, key: bytes, value: bytes) -> bool {
	sync.shared_lock(c.lock)
	defer sync.shared_unlock(c.lock)
	return false
}
