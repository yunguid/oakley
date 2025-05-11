# Oakley SRS

Oakley SRS is an **offline, AI-powered spaced-repetition system** that captures knowledge snippets from any screen, turns them into flash-cards with a local LLM, and periodically quizzes the user by voice or text.

## Features

- Screenshot knowledge capture with one hotkey (⇧⌘S)
- Automatic flashcard generation with local LLM
- Spaced repetition scheduling with SM-2 algorithm
- Voice and text-based review
- Complete privacy - runs 100% offline

## Development Status

This project is currently in early development stage (pre-alpha).

## Quick Start

### Building from Source

Prerequisites:
- Rust 1.70+
- Cargo

Clone and build:

```bash
git clone https://github.com/yourusername/oakley-srs.git
cd oakley-srs
make build
```

Run the application:

```bash
make run
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
