# PostureWatch

PostureWatch is a privacy-first, fully implemented posture monitoring and alert application written in Rust.
It periodically checks your posture using your webcam and an LLM endpoint, falling back to a local heuristic if needed. It also features a repeating blink alert system to help correct bad posture, and a time-based reminder to raise your standing desk or stretch.

## Fully Implemented Features

- **Periodic Webcam Capture:** Captures snapshots efficiently using `nokhwa`.
- **Posture Analysis:** Uses a configurable Vision LLM endpoint to analyze posture (e.g. OpenAI GPT-4 Vision).
- **Local Fallback:** Falls back gracefully when the API is unavailable or privacy mode is enabled.
- **Repeating Alert:** Notifies continuously until posture improves via cross-platform system notifications.
- **Desk Raise Reminder:** Time-based recurring alerts to stand up.
- **Persistent Configuration:** Fully configurable via a local `config.toml`.
- **Privacy Mode:** Ensures minimal data is sent or uses pure local fallback to maintain privacy.

## Roadmap

- Remove "privacy mode" and "local fallback" 
- Simplify code and architecture
- Code and prompts review
- Break reminder with repeating alert reminder
- Legal note, friendly installer and MVP publish
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
- LLM API Key (optional, if disabled will use local fallback)

### Run
```bash
cargo run --release
```

### Config
The configuration file is automatically generated on first run. 
Edit `config.toml` to set:
- `api_key`: Your LLM provider API key (default: empty).
- `privacy_mode`: Set to `true` to use only local fallback.
- `cycle_time_secs`: How often to check posture.
- `desk_raise_interval_secs`: How often to remind to raise the desk.

## Limitations
- Local fallback currently defaults to "Unknown" because a robust local lightweight vision model is not bundled directly within this binary yet.
- "Corner blink alert" currently utilizes the OS's native notification system (which usually pops up in the corner).

## Troubleshooting
- **No webcam found:** Check permissions for `/dev/video0` or your equivalent OS camera permissions.
- **Missing dependencies (Linux):** Install `libclang-dev`, `libavcodec-dev`, `libavformat-dev`, etc., as noted in `.github/workflows/rust.yml`.
