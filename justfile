build:
	cargo build
run:
  cargo run
test:
  cargo test -- --nocapture

issuepack:
  uv run --no-project ~/programming/issuepack/issue.py

