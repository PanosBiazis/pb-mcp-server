# main.rs patch instructions

Two manual edits needed in `src/main.rs` after merging this branch:

## 1. Add mod declarations at the top of main.rs (after existing `mod` block)

```rust
mod tool_args_wired;  // arg structs for wired tools
```

And add these use-imports in the `use crate::ops::{ ... };` block — they are already
there from the original code, so no change needed.

Add this import alongside the existing `use crate::ops::` block:
```rust
use crate::tool_args_wired::*;
```

## 2. Add `include!` at the END of the `#[tool_router] impl MintMcp { }` block

Find the very last `}` that closes `#[tool_router] impl MintMcp {` (search for
the pattern `} // end of #[tool_router] impl` or just the final closing brace of
the enormous impl block), and **before** that closing `}` add:

```rust
    // --- Wired tools: all previously unused *_work imports ---
    include!("tools_wired.rs");
```

This splices every `#[tool]` method from `src/tools_wired.rs` directly into
the impl block, making them visible to the rmcp `#[tool_router]` macro.

## 3. Build

```bash
cargo build --release
# Expect: zero "unused import" warnings for the groups listed in tools_wired.rs
# Zed tools/list will grow by ~29 new tools
```

## Why include! instead of a separate impl block?

The `#[tool_router]` macro on `impl MintMcp` must see ALL `#[tool]` methods in
one syntactic impl block. A second `impl MintMcp { ... }` with its own
`#[tool_router]` would create a second independent router that never gets
merged into `self.tool_router`. The `include!()` macro textually pastes the
methods inside the *same* `#[tool_router] impl MintMcp` block, which is the
correct approach used by rmcp-based servers.
