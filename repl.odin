package main

import "core:bytes"
import "core:fmt"
import "core:os"

resp_result :: enum {
	OK,
	INVALID_COMMAND,
	KEY_DOES_NOT_EXIST,
}

repl_loop :: proc(c: ^cache) {


	for {
		buf: [256]u8
		fmt.printf(">> ")
		n, err := os.read(os.stdin, buf[:])
		if err != nil || n == 0 {
			break
		}
		buf[n - 1] = 0
		resp, res := repl_parse(c, buf[:])
		if res == resp_result.OK {
			fmt.printf("OK %s\n", resp)
		} else {
			fmt.printf("%s\n", res)
		}
	}
}


@(private)
count_of_non_zero_bytes :: proc(b: []u8) -> int {
	c := 0
	for i := 0; i < len(b); i += 1 {
		if b[i] == 0 do continue
		c += 1
	}
	return c
}

repl_parse :: proc(c: ^cache, buf: []u8) -> ([]u8, resp_result) {
	if count_of_non_zero_bytes(buf) < 5 {
		return []u8{}, resp_result.INVALID_COMMAND
	}
	new := bytes.trim_space(buf)
	switch string(new[:3]) {
	case "GET":
		key := new[4:count_of_non_zero_bytes(new)]
		resp, ok := cache_get(c, key).?
		if ok {
			return resp, resp_result.OK
		}
		return key, resp_result.KEY_DOES_NOT_EXIST
	case "SET":
		split := bytes.split_n(new[4:], []u8{32}, 2)
		ok := cache_set(c, split[0], split[1], -1)
		return []u8{}, resp_result.OK
	case "DEL":
		key := new[4:count_of_non_zero_bytes(new)]
		ok := cache_delete(c, key)
		return []u8{}, resp_result.OK
	}

	return []u8{}, resp_result.INVALID_COMMAND
}
