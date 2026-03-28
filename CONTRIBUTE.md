# Contributing

## Working together

Please be kind, constructive, and respectful. This project is open source, and a friendly tone makes it easier for people to contribute. If you would like to make an non-trivial contribution please open an issue beforehand.

## Architecture
- **Camera module:** Captures JPEG-encoded RGB frames from the system webcam.
- **Posture analyzer:** Sends frames to a remote vision LLM (for example, OpenAI) via `reqwest` and parses only `"Good"` or `"Bad"`.
- **Monitor logic:** Uses a state machine to track consecutive bad-posture events and alert timing.
- **Alert system:** Sends desktop notifications with `notify-rust`.
- **Config:** Loads and saves settings with `directories` in the standard OS config location (typically `~/.config/posturewatch/config.toml`).

## Setup
### Prerequisites
- Rust + Cargo (`curl https://sh.rustup.rs -sSf | sh`)
- Webcam
- LLM API key

### Run
```bash
cargo run --release
```

## Configuration
`config.toml` is generated on first run. Typical fields:
- `api_key`: LLM provider API key (default: empty)
- `model`: Vision model name (default: `gpt-5.4-mini`)
- `cycle_time_secs`: Posture check interval
- `desk_raise_interval_secs`: Desk-raise reminder interval

## Quality checks
Before opening a PR, run:
```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build
```
