package main

import "core:fmt"


dbg :: proc() {
	m := make(map[string]int)
	fmt.printf("%v \n", m["a"])
	m["a"] = 1
	fmt.printf("%v \n", m["a"])
}
