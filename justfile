build:
	cargo build --release
run:
  cargo run --release
test:
  cargo test --release --color always --message-format human -- --nocapture --test-threads=1
