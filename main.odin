package main

import "core:fmt"
import "core:math/rand"


main :: proc() {
	defer free_all(context.temp_allocator)
	c := cache_create(10_000_000)
	defer {
		cache_free(&c)
	}


	for i: u128 = 0; i < 10; i += 1 {
		key := fmt.tprintf("key-%03x-%d", rand.uint64(), i)
		value := fmt.tprintf("val-%03x-%d", rand.uint64(), i)
		cache_set(&c, transmute([]u8)key, transmute([]u8)value, -1)
	}
	k, v := "hello", "world"
	cache_set(&c, transmute([]u8)k, transmute([]u8)v, -1)
	k, v = "2", "world"
	cache_set(&c, transmute([]u8)k, transmute([]u8)v, -1)

	for item, i in c.items {
		fmt.printf("%d\t%s\n", i, c.items[i].bytes[:])
	}
	v2 := "thisis\\talongervalu\\0e"
	fmt.println()
	cache_set(&c, transmute([]u8)k, transmute([]u8)v2, -1)
	for item, i in c.items {
		fmt.printf("%d %s\n", i, c.items[i].bytes[:])
	}
	fmt.println()


	value, ok := cache_get(&c, transmute([]u8)k).?
	if ok do fmt.printf("%s %s\n", k, value)


	we := cache_get(&c, []u8{})
	fmt.printf("%v\n", we)
}
