# Agent Instructions

1. Read `CONTRIBUTE.md` before making changes.
2. Keep edits minimal, focused, and consistent with existing project style.
3. Run standard local checks before finalizing:
   - `cargo fmt`
   - `cargo clippy -- -D warnings`
   - `cargo test`
   - `cargo build`
4. When polling GitHub Actions CI, use exponential backoff in seconds:
   `1, 3, 5, 10, 30, 60, 120, 180, 180, timeout`.
