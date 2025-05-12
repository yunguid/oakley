# Oakley SRS

Oakley SRS is an AI-powered spaced-repetition system that captures text or screenshots from your screen, turns them into flash-cards using OpenAI, and periodically quizzes the user by voice or text.

## Features

- Text selection capture (⇧⌘>)
- Screenshot capture (⇧⌘<)
- Automatic flash-card generation through OpenAI
- Spaced repetition scheduling with SM-2 algorithm
- Voice and text-based review
- Web interface for browsing cards (localhost:5173)
- REST API for card access (localhost:3030)

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

# Use ⇧⌘> to capture selected text
# Use ⇧⌘< to capture screenshot
# Browse cards at http://localhost:5173
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

- `capture`: Screen capture and text selection
- `llm`: OpenAI integration for card generation
- `scheduler`: Spaced repetition algorithm (SM-2)
- `data`: Database operations and REST API
- `utils`: Shared utilities
- `oakley-cli`: Command-line interface and orchestration

## License

MIT or Apache-2.0, at your option.
