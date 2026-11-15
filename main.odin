package main

import "core:fmt"


main :: proc() {
	c := cache_create(10_000_000)
	fmt.printf("Cache created with %d items\n", len(c.items))
	defer {
		fmt.printf("Destroying cache with %d items...\n", len(c.items))
		cache_free(&c)
		fmt.printf("Cache destroyed\n")
	}

	// set and get

  _ := cache_set(b"test", b"123")


}
