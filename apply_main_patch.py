#!/usr/bin/env python3
"""
apply_main_patch.py — run from repo root: python3 apply_main_patch.py
Applies the 3 wiring insertions to src/main.rs in-place.
"""
import re, sys, pathlib

path = pathlib.Path("src/main.rs")
if not path.exists():
    sys.exit("ERROR: src/main.rs not found. Run from repo root.")

text = path.read_text(encoding="utf-8")
original = text

# ── Patch 1: add `mod tool_args_wired;` after `mod thinking;` ────────────────
if "mod tool_args_wired;" not in text:
    text = text.replace(
        "mod thinking;",
        "mod thinking;\nmod tool_args_wired;",
        1
    )
    print("✓ Patch 1 applied: mod tool_args_wired;")
else:
    print("⚠ Patch 1 already present, skipping.")

# ── Patch 2: add `use crate::tool_args_wired::*;` after thinking use block ───
# The thinking use block ends with `thinking_step_back_work,\n};`
THINKING_END = "    thinking_step_back_work,\n};"
if "use crate::tool_args_wired::*;" not in text:
    if THINKING_END in text:
        text = text.replace(
            THINKING_END,
            THINKING_END + "\nuse crate::tool_args_wired::*;",
            1
        )
        print("✓ Patch 2 applied: use crate::tool_args_wired::*;")
    else:
        print("✗ Patch 2 FAILED: could not find thinking use block end")
        sys.exit(1)
else:
    print("⚠ Patch 2 already present, skipping.")

# ── Patch 3: add include!("tools_wired.rs"); before final `}` of file ────────
# The #[tool_router] impl MintMcp ends at the very last `}` in the file.
# We find the last occurrence of `\n}` and insert before it.
if 'include!("tools_wired.rs");' not in text:
    last_brace_pos = text.rfind("\n}")
    if last_brace_pos == -1:
        print("✗ Patch 3 FAILED: could not find final closing brace")
        sys.exit(1)
    text = (
        text[:last_brace_pos]
        + '\n\n    // ── Wired tools: all previously-unused *_work imports ──────────────────────\n'
        + '    include!("tools_wired.rs");\n'
        + text[last_brace_pos:]
    )
    print('✓ Patch 3 applied: include!("tools_wired.rs");')
else:
    print("⚠ Patch 3 already present, skipping.")

if text == original:
    print("\nNo changes made (all patches already present).")
else:
    path.write_text(text, encoding="utf-8")
    print(f"\nDone. src/main.rs updated ({len(text)} bytes).")
    print("Run: cargo check 2>&1 | grep 'unused import'")
    print("Expected: zero unused-import warnings for the wired tool groups.")
