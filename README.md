# PostureWatch

[![CI & Release](https://github.com/lextiz/PostureWatch/actions/workflows/rust.yml/badge.svg)](https://github.com/lextiz/PostureWatch/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/lextiz/PostureWatch/graph/badge.svg)](https://codecov.io/gh/lextiz/PostureWatch)

PostureWatch is a small desktop app that helps you notice bad posture and remember to take breaks while you work at a screen.

Many of us spend long hours at a desk these days, and posture usually gets worse long before we notice it.

It uses your webcam together with a vision model to check posture from time to time, then nudges you when it looks like it’s time to sit better, stand up, or step away from the desk.

This project is open source because better desk habits should be easier to build, improve, and share.

## What it does

- Periodically checks posture using a configurable vision model
- Shows persistent reminders until posture improves
- Reminds you to raise your desk or take a break
- Runs quietly in the system tray

## Getting started

1. Download the latest `PostureWatch.exe` from the [Releases](https://github.com/lextiz/PostureWatch/releases) page.
2. Start `PostureWatch.exe`. It will appear in your system tray.
3. Open **Configure...** from the tray menu.
4. Add an API key for a supported AI provider.
5. Let it run quietly in the background while you work.

At the moment, OpenAI is the supported provider.

## Release trust and OS security checks

Official release artifacts are produced in GitHub Actions and signed before upload:

- **Windows:** `PostureWatch.exe` is Authenticode-signed with a timestamp.
- **macOS:** `PostureWatch-macos` is code-signed, and `PostureWatch-macos.zip` is submitted for notarization.

If a build is distributed outside the official Releases page or without these steps, Windows SmartScreen and macOS Gatekeeper can flag it as suspicious.

## Privacy and trust

PostureWatch uses your webcam and sends posture checks to the vision provider you configure, so please use a provider you trust and review its privacy terms.

It is built to help with posture and breaks, not surveillance or invasive use. Please use it lawfully, with appropriate consent where needed.

Posture checks can cost API tokens, so infrequent checks and reasonably priced models are usually enough.

## Contributing

If you’d like to help, please see [contribute.md](./contribute.md).

If this is useful to you, a GitHub star helps more people find it.

## Roadmap

- Detection of absence and exponential backoff (eventually auto-pause) to save tokens
- Option to configure maximum number of repititions of the presistent reminder
- Icon should reflect monitoring status
- Specific hint in notification, e.g.: relax your shoulders, keep your head back
- Screen frame warning (instead of notifications)
- OS portability: MacOS, Linux
- Local model
