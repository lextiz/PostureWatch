# PostureWatch

PostureWatch is a fully implemented posture monitoring and alert application written in Rust.
It periodically checks your posture using your webcam and an LLM endpoint. It also features a repeating blink alert system to help correct bad posture, and a time-based reminder to raise your standing desk or stretch.

## Fully Implemented Features

- **Periodic Webcam Capture:** Captures snapshots efficiently using `nokhwa`.
- **Posture Analysis:** Uses a configurable Vision LLM endpoint and model name to analyze posture.
- **Repeating Alert:** Notifies continuously until posture improves via cross-platform system notifications.
- **Desk Raise Reminder:** Time-based recurring alerts to stand up.
- **Persistent Configuration:** Fully configurable via a local `config.toml`.

## Roadmap

- Simplify code and architecture
- Code and prompts review
- Break reminder with repeating alert reminder
- Legal note, friendly installer and MVP publish
- Screen frame warning
- Prompts customization
- OS portability
- Local model campagin

## Architecture

1. **Camera Module:** Interfaces with the system webcam to capture JPEG-encoded RGB frames.
2. **Posture Analyzer:** Communicates with remote Vision LLMs (e.g., OpenAI) via `reqwest`, parsing strictly for "Good" or "Bad" posture.
3. **Monitor Logic:** An abstracted state machine that tracks consecutive bad posture events and manages the alert logic cleanly.
4. **Alert System:** Triggers notifications via `notify-rust`.
5. **Config:** Loads and persists user settings via the `directories` crate in `~/.config/posturewatch/config.toml` (or standard config path per OS).

## Setup & Running

### Prerequisites
- Rust and Cargo (`curl https://sh.rustup.rs -sSf | sh`)
- A webcam
- LLM API Key

### Run
```bash
cargo run --release
```

### Config
The configuration file is automatically generated on first run. 
Edit `config.toml` to set:
- `api_key`: Your LLM provider API key (default: empty).
- `model`: Vision-capable model name (default: `gpt-5.4-mini`).
- `cycle_time_secs`: How often to check posture.
- `desk_raise_interval_secs`: How often to remind to raise the desk.

## Limitations
- "Corner blink alert" currently utilizes the OS's native notification system (which usually pops up in the corner).

## Troubleshooting
- **No webcam found:** Check permissions for `/dev/video0` or your equivalent OS camera permissions.
- **Missing dependencies (Linux):** Install `libclang-dev`, `libavcodec-dev`, `libavformat-dev`, etc., as noted in `.github/workflows/rust.yml`.
