# Twin

**Privacy-First AI Meeting Assistant**

Twin is an open-source, privacy-first AI meeting assistant that captures, transcribes, and summarizes meetings entirely on your local machine. No cloud. No data leaks. Complete control.

---

## Overview

Twin runs entirely on your device, using local AI models to transcribe meetings in real-time and generate intelligent summaries. Perfect for professionals and enterprises who need meeting intelligence without compromising privacy, compliance, or data sovereignty.

**Key Principles:**
- **Privacy First** - All processing happens locally on your device
- **Cost-Effective** - Uses open-source AI models instead of expensive APIs
- **Flexible** - Works offline and supports multiple meeting platforms
- **Customizable** - Self-host and modify for your specific needs

---

## Features

- **Real-time Transcription** - Live transcripts as your meeting happens
- **AI-Powered Summaries** - Generate meeting summaries using powerful language models
- **Local First** - No data ever leaves your computer
- **Multi-Platform** - Works on macOS, Windows, and Linux
- **Flexible AI Provider Support** - Choose from Ollama (local), Claude, Groq, OpenRouter, or your own OpenAI-compatible endpoint
- **Import & Enhance** - Import existing audio files to generate or re-transcribe meetings
- **Professional Audio Mixing** - Capture microphone and system audio simultaneously with intelligent ducking
- **GPU Acceleration** - Built-in support for Metal (macOS), CUDA (NVIDIA), and Vulkan (AMD/Intel)

---

## Installation

### Windows

1. Download the latest `x64-setup.exe` from [Releases](https://github.com/kanishka-namdeo/Twin/releases/latest)
2. Run the installer

### macOS

1. Download `twin_0.4.0_aarch64.dmg` from [Releases](https://github.com/kanishka-namdeo/Twin/releases/latest)
2. Open the downloaded `.dmg` file
3. Drag **Twin** to your Applications folder
4. Open **Twin** from Applications folder

### Linux

Build from source:

```bash
git clone https://github.com/kanishka-namdeo/Twin
cd Twin/frontend
pnpm install
./build-gpu.sh
```

See [Building on Linux](docs/building_in_linux.md) and [General Build Instructions](docs/BUILDING.md) for detailed guides.

---

## System Architecture

Twin is a single, self-contained application built with [Tauri](https://tauri.app/). It uses a Rust-based backend for core logic and a Next.js frontend for the user interface.

For more details, see the [Architecture documentation](docs/architecture.md).

---

## For Developers

To contribute or build from source, you'll need Rust and Node.js installed. See the [Building from Source guide](docs/BUILDING.md) for detailed instructions.

---

## Contributing

Contributions are welcome! Open an issue or submit a pull request. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT License - Free to use for your own purposes.

---

## Acknowledgments

- [Whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- [Screenpipe](https://github.com/mediar-ai/screenpipe)
- [transcribe-rs](https://crates.io/crates/transcribe-rs)
- **NVIDIA** for the **Parakeet** model
- [istupakov](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx) for the ONNX conversion of Parakeet

---

## Links

- [GitHub](https://github.com/kanishka-namdeo/Twin)
