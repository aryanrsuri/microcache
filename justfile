build:
	cargo build
run:
  cargo run
test:
  cargo test --color always --message-format human -- --nocapture --test-threads=1
