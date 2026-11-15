package main

MAX_KEY_LEN: uint : 255
MAX_VAL_LEN: uint : 1_000_000

item :: struct {
	prev:        int,
	next:        int,
	accessed_at: u64,
	expires_at:  u64,
	key_len:     int,
	val_len:     int,
	bytes:       bytes,
}
