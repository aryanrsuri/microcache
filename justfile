build:
	cargo build
run:
  cargo run
test:
  cargo test can_perf -- --ignored --nocapture

issuepack:
  uv run --no-project ~/programming/issuepack/issue.py

