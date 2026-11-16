package main

import "core:sync"


// MUST Not be copied after first use

cache :: struct {
	lock:       sync.Mutex,
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
	sync.mutex_lock(&c.lock)
	defer sync.mutex_unlock(&c.lock)


	if c.newest == -1 {
		item, used := item_create(key, value, -1, -1, expires_at)
		assign_at(&c.items, 0, item)
		c.index[cast(string)item.bytes[:item.key_len]] = 0
		c.newest = 0
		c.oldest = 0
		c.used_bytes += used
		return true
	}

	index, exists := c.index[cast(string)key]
	if exists {
		item := &c.items[index]
		delete_key(&c.index, string(item.bytes[:item.key_len]))
		used := item_update_value(item, value, expires_at)
		c.index[string(item.bytes[:item.key_len])] = index
		c.used_bytes += used
	} else {
		c.newest += 1
		item, used := item_create(key, value, -1, -1, expires_at)
		assign_at(&c.items, c.newest, item)
		c.index[cast(string)item.bytes[:item.key_len]] = c.newest
		c.used_bytes += used
	}

	return true

}

cache_get :: proc(c: ^cache, key: []u8) -> Maybe([]u8) {
	sync.mutex_lock(&c.lock)
	defer sync.mutex_unlock(&c.lock)

	if c.newest == -1 {
		return nil
	}

	index, exists := c.index[cast(string)key]
	if exists {
		item := &c.items[index]
		value := make([]u8, item.val_len)
		copy_slice(value, item.bytes[item.key_len:item.val_len])

		return value
	}


	return nil
}
