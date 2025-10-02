# Changelog

## Closures

### Upvalue Compilation
- Reworked `CompileFrame` to store per-function upvalue metadata and emit `Value::Function` constants so closure operands match the VM's expectations.
- Taught `Parser::resolve_upvalue`/`named_variable` to walk outward scopes, deduplicate captures, and compile `OP_GET_UPVALUE`/`OP_SET_UPVALUE` operands with real indices.

### VM Runtime Support
- Added opcode mappings for `OP_GET_UPVALUE`/`OP_SET_UPVALUE` and implemented closure instantiation that wires captured locals into `Closure` values.
- Hooked `CallFrame`/`Call` sites up to the new closure representation so nested functions execute without hitting "Invalid function object" errors.

### Validation
- Ran `cargo run -- test.lox` (with and without `trace_execution`) to confirm the program now prints `outside`.

## Slot

### Stack Slot Handling Refactor
- Updated `src/vm.rs` call-frame representation to store a `slot_base` index instead of a cloned slot vector, ensuring locals are always read from the VM stack.
- Taught the VM to fetch and write locals via stack indexing with runtime bounds checks, fixing the `OP_GET_LOCAL` panic triggered by `test.lox`.
- Adjusted call and return behavior so frames share a single stack: native calls capture arguments from the stack, new frames inherit a stack window, and returns truncate back to the caller before pushing results.

### Parser Local Tracking
- Seeded compile frames (script and functions) with synthetic local slots to align with the VM’s slot layout and preserve `this`/closure slot semantics.
- Corrected reverse-iteration bookkeeping in `Parser::resolve_local` so it reports the true local index and flags reads from uninitialized locals reliably.

### Validation
- Ran `cargo fmt` to keep formatting consistent.
- Verified `cargo run -- test.lox` now prints `global` without panicking.
- Executed `cargo test` (no tests defined yet) to confirm the crate still compiles cleanly.
