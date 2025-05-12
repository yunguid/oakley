# Oakley SRS

Oakley SRS is an **offline, AI-powered spaced-repetition system** that captures knowledge snippets from any screen, turns them into flash-cards with a local LLM, and periodically quizzes the user by voice or text.

## Features

- Clipboard capture with one hotkey (⇧⌘P)
- Automatic flash-card generation through OpenAI (local LLM WIP)
- Spaced repetition scheduling with SM-2 algorithm
- Voice and text-based review
- Complete privacy - runs 100% offline

## Development Status

This project is currently in early development stage (pre-alpha).

## Quick Start

### Dev loop (Mac/Linux)

Prereqs
* Rust ≥ 1.74
* Node 18 + pnpm / npm
* OpenAI key in env `OPENAI_API_KEY`

```bash
git clone https://github.com/yourusername/oakley-srs.git
cd oakley-srs

# first run – install Rust + JS deps
cargo check -p oakley-tauri           # pulls crates
cd tauri-app && npm i                 # pulls front-end deps

# dev run (backend + front-end hot-reload, logs in same terminal)
cd tauri-app
RUST_LOG=debug npm run tauri dev

# press ⇧⌘P to convert clipboard → flash-card
```

### Release build

```bash
cd tauri-app
npm run tauri build   # produces .app / .dmg / .msi in target/release/bundle
```

## Development

Common tasks:

```bash
make check      # Run lints
make test       # Run tests
make fix        # Fix warnings
make clean      # Clean build artifacts
make build-full # Build with all features 
```

## Architecture

Oakley is built as a Rust workspace with these main components:

- `capture`: Screen capture and region selection
- `ocr`: Optical character recognition
- `llm`: Local language model for card generation
- `scheduler`: Spaced repetition algorithm (SM-2)
- `data`: Database operations
- `utils`: Shared utilities
- `oakley-cli`: Command-line interface and orchestration

## License

MIT or Apache-2.0, at your option.
