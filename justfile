build:
  cargo build

# Test everything
test: build
  cargo nextest run