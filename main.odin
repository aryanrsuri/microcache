package main

import "core:fmt"
import "core:math/rand"


main :: proc() {
	defer free_all(context.temp_allocator)
	c := cache_create(10_000_000)
	fmt.printf("Cache created with %d items\n", len(c.items))
	defer {
		fmt.printf("Destroying cache with %d items...\n", len(c.items))
		cache_free(&c)
		fmt.printf("Cache destroyed\n")
	}


	for i: u128 = 0; i < 1_000; i += 1 {
		key := fmt.tprintf("key-%016x-%d", rand.uint64(), i)
		value := fmt.tprintf("val-%016x-%d", rand.uint64(), i)
		cache_set(&c, transmute([]u8)key, transmute([]u8)value, -1)
	}

	fmt.printf("Cache created with %d items\n", len(c.items))
	for item, i in c.items {
		fmt.printf("%d...%s\n", i, c.items[i].bytes[:])
	}
}
