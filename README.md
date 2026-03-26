# PostureWatch

PostureWatch is a basic posture/screen time monitoring and alert application.
It periodically checks your posture using your webcam and an LLM endpoint. If your posture degrades or you did not have a break for too long you are forced to improve with annoying alerts.

## Features

- **Posture Analysis:** Uses a configurable Vision LLM endpoint and model name to analyze posture.
- **Insisting Posture Alert:** Notifies continuously until posture improves.
- **Desk Raise Reminder:** Time-based recurring alerts to stand up.

## Usage

- Download the [latest released installation file](https://github.com/lextiz/PostureWatch/releases)
- Run the programm, it will appear in tray
- Right click -> options -> configure an AI provider API key (*currently only OpenAI is supported, you can only get an API key if you have a developer account*)
- Enjoy

## Roadmap

- Simplify code and architecture
- Code and prompts review
- Break reminder with repeating alert reminder
- Legal note, friendly installer and MVP publish
- Screen frame warning
- Prompts customization
- OS portability
- Local model campagin
