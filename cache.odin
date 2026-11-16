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
	used_bytes: int,
	max_bytes:  int,
}

cache_create :: proc(max_bytes: uint) -> cache {
	return {
		index      = make(map[string]int),
		items      = make([dynamic]item),


		// Ignoring the below for now
		free       = make([dynamic]int),
		newest     = -1,
		oldest     = -1,
		used_bytes = 0,
		max_bytes  = cast(int)max_bytes,
	}
}

cache_free :: proc(c: ^cache) {
	for &item, _ in c.items {delete_dynamic_array(item.bytes)}
	delete_dynamic_array(c.items)
	delete_dynamic_array(c.free)
	delete_map(c.index)
}

cache_set :: proc(c: ^cache, key: []u8, value: []u8, expires_at: i64) -> bool {
	sync.shared_lock(&c.lock)
	defer sync.shared_unlock(&c.lock)


	// put a new item at the head of the items array
	// store the index of the new item in index
	// update newest & oldest
	// update used_bytes
	// repack if max_bytes is close to full

	str := cast(string)key
	if c.newest == -1 {
		item, used := item_create(key, value, -1, -1, -1)
		inject_at(&c.items, 0, item)
		c.index[str] = 0
		c.newest = 0
		c.oldest = 0
		c.used_bytes += used
		return true
	}

	index, exists := c.index[str]
	if exists {
		item := c.items[index]
		used := item_update_value(&item, key, expires_at)
		c.used_bytes += used
	} else {
		c.newest += 1
		item, used := item_create(key, value, -1, -1, -1)
		c.items[c.newest] = item
		c.index[str] = c.newest
		c.used_bytes += used
	}

	return true

}
