build:
  cargo build

# Test everything
test:
  cargo nextest run

format:
  cargo fmt --all

lint:
  cargo clippy --all --all-targets --all-features -- -D warnings

lintfix:
  cargo clippy --fix --allow-staged --allow-dirty

refresh:
  cargo clean && cargo update