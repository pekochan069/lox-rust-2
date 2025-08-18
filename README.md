# Lox VM in Rust

Implementation of Lox interpreter with vm in rust.

Check out [Crafting Interpreters](https://craftinginterpreters.com/) for more information about the language.

## Build Project

### Requirements

1. Rust Compiler

Install Rust compiler using [rustup](https://rustup.rs/)

```bash
rustup install nightly
rustup default nightly
```

### Clone Project

```bash
git clone https://github.com/pekochan069/lox-rust-2
cd lox-rust-2
```

### Build using cargo

```bash
cargo build
```

To enable debug disassembly, use `disassemble` feature

```bash
cargo build --features disassemble
```
