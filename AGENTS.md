# PostureWatch Contributor Guide

## Development Setup

### Prerequisites
- Rust (stable toolchain)
- Linux: `sudo apt-get install libclang-dev clang libavcodec-dev libavformat-dev libswscale-dev libavutil-dev libglib2.0-dev libgtk-3-dev libgdk-pixbuf2.0-dev libv4l-dev`

## Code Quality

### Before Pushing
**Always run the following commands locally before pushing to ensure all CI checks pass:**

```bash
# Run all local checks
cargo fmt -- --write
cargo clippy -- -D warnings
cargo test
cargo build
```

If any command fails, fix the issues and re-run until all pass. You can make multiple local commits for readability, but ensure the final push contains only passing checks.

### CI Checks
The CI pipeline runs these jobs in parallel:
- **fmt**: Checks code formatting (`cargo fmt -- --check`)
- **clippy**: Lints code with clippy (`cargo clippy -- -D warnings`)
- **test**: Runs unit tests (`cargo test`)
- **build**: Builds the project (`cargo build`)
- **build-release**: Builds release artifacts and creates GitHub releases (on release events)

### Best Practices
- Always format code with `cargo fmt` before committing
- Address clippy warnings (they're treated as errors in CI)
- Ensure tests pass before pushing
- For release builds, create a tag following the `v*` pattern (e.g., `v0.1.0`)
