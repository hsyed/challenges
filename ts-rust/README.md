# Rust + TypeScript + Deno FFI Example

A complete example demonstrating seamless integration between Rust and TypeScript using Deno's Foreign Function Interface (FFI).

## 🚀 Quick Start

```bash
./test.sh
```

## 📁 Project Structure

```
ts-rust/
├── src/lib.rs          # Rust library with unsafe C-compatible functions
├── Cargo.toml          # Rust configuration for dynamic library
├── main.ts             # Deno TypeScript application using FFI
├── deno.json           # Deno configuration with FFI types
├── test.sh            # Automated build and run script
└── README.md           # This documentation
```

## 🔧 Functions

| Function | Purpose | Memory Management |
|----------|---------|-------------------|
| `add_numbers` | Integer arithmetic | Stack-based |
| `greet` | String manipulation | Heap-allocated, requires `free_string` |
| `free_string` | Memory cleanup | Deallocates Rust-allocated strings |
| `fibonacci` | Mathematical computation | Stack-based |
| `sum_array` | Array processing | Read-only view, no ownership transfer |

## 🛡️ Safety

All pointer-accepting functions are marked `unsafe` with documented safety requirements:

```rust
/// # Safety
/// `name` must be a valid pointer to a null-terminated C string.
pub unsafe extern "C" fn greet(name: *const c_char) -> *mut c_char
```

## 🏗️ Build Process

**Automatic:**

```bash
./test.sh
```

**Manual:**

```bash
# Build Rust dynamic library
cargo build --release

# Run Deno with FFI permissions
deno run --allow-ffi --unstable-ffi main.ts
```

## 📋 Prerequisites

- [Rust](https://rustup.rs/) toolchain
- [Deno](https://deno.land/) runtime

## 🔍 Key Concepts

- **FFI Bindings**: Type-safe bridges between Rust and JavaScript
- **Memory Safety**: Explicit ownership transfer for heap-allocated data  
- **Unsafe Functions**: Raw pointer operations require careful handling
- **Permission Model**: Deno's security requires explicit FFI allowance

## ⚠️ Important Notes

- String functions transfer ownership - caller must free memory
- Array functions use read-only views - no cleanup needed
- All FFI operations require `--allow-ffi --unstable-ffi` flags
- Pointer validity is caller's responsibility
