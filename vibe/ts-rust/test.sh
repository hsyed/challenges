#!/usr/bin/env bash

echo "🔨 Building Rust library..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Rust library built successfully!"
    echo ""
    echo "🚀 Running Deno app..."
    deno run --allow-ffi --unstable-ffi main.ts
else
    echo "❌ Failed to build Rust library"
    exit 1
fi