#!/bin/bash
set -e

echo "Building WASM with manual cargo + wasm-bindgen..."

# Navigate to the wasm directory
cd ../wasm

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Build with RUSTFLAGS for getrandom
echo "Building WASM binary..."
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' \
  cargo build --target wasm32-unknown-unknown --release

# Check if the WASM file was created (in workspace target directory)
cd ..
WASM_FILE="target/wasm32-unknown-unknown/release/dc_api_wasm.wasm"
if [ ! -f "$WASM_FILE" ]; then
    echo "Error: WASM file not found at $WASM_FILE"
    exit 1
fi

# Create output directory
mkdir -p npm-package/dist

# Generate JavaScript bindings with wasm-bindgen
echo "Generating JavaScript bindings..."
wasm-bindgen \
  --out-dir npm-package/dist \
  --target nodejs \
  --typescript \
  "$WASM_FILE"

echo "✅ WASM build completed successfully!"
echo "Generated files:"
ls -la npm-package/dist/
