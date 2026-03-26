# PostureWatch

[![CI & Release](https://github.com/lextiz/PostureWatch/actions/workflows/rust.yml/badge.svg)](https://github.com/lextiz/PostureWatch/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/lextiz/PostureWatch/graph/badge.svg)](https://codecov.io/gh/lextiz/PostureWatch)

PostureWatch is a small desktop app that helps you notice bad posture and remember to take breaks while you work at PC.

It uses your webcam together with a vision model to check posture from time to time, then nudges you when it looks like it’s time to sit better, stand up, or step away from the desk.

This project is open source because better desk habits should be easier to build, improve, and share.

## What it does

- Periodically checks posture using a configurable vision-model
- Shows persistent reminders until posture improves
- Reminds you to raise your desk or take a break
- Runs quietly in the system tray

## Getting started

1. Download the latest installer from the [Releases](https://github.com/lextiz/PostureWatch/releases) page.
2. Start PostureWatch. It will appear in your system tray.
3. Open **Configure...** from the tray menu.
4. Add an API key for a supported AI provider.
5. Let it run quietly in the background while you work.

At the moment, OpenAI is the supported provider.

## Privacy and common-sense disclaimer

PostureWatch uses your webcam and sends posture checks to the vision endpoint you configure, so please use a provider you trust and review its privacy terms.

PostureWatch is here to help with posture and breaks, not surveillance or invasive use. Please use it lawfully, with appropriate consent where needed.

## Contributing

If you’d like to help, please see [contribute.md](./contribute.md).

Contributions of all sizes are welcome.

If you like it, a GitHub star is appreciated.

## Roadmap

- Simplify code and architecture
- Code and prompts review
- Break reminder with repeating alert reminder
- Legal note, friendly installer and MVP publish
- Icon should reflect monitoring status, tooltip: time at PC (today, current session), current posture score
- Screen frame warning
- Prompts customization
- OS portability
- Local model
