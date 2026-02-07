# AI Coding Agent Instructions for epub_download

## Project Overview

A Tauri-based desktop application for downloading web novels from bilinovel.com and generating EPUB e-books. Supports content decryption, automatic cover/catalog generation, and runs in both GUI and CLI modes.

## Architecture

### Dual-Mode Design
- **GUI Mode** (default): Tauri desktop app with Vue.js frontend
- **CLI Mode**: Command-line tool via `src-tauri/src/bin/cli.rs`
- Feature gating: Use `#[cfg(feature = "gui")]` for GUI-only code

### Core Components

**Backend (Rust - `src-tauri/src/`):**
- `client.rs`: HTTP client with custom headers for bilinovel.com
- `downloader.rs`: Main download orchestrator, handles chapter fetching and EPUB generation
- `epub_builder.rs`: EPUB file construction with metadata, content blocks, and images
- `parse.rs`: HTML parsing using `scraper` crate for book info and volume lists
- `paragraph_restorer.rs`: Restores scrambled paragraph order using Fisher-Yates algorithm
- `secret.rs`: Character mapping for content decryption (7000+ character pairs)
- `manage.rs`: EPUB library management, builds index from existing files
- `config.rs`: JSON configuration with defaults
- `command.rs`: Tauri commands exposed to frontend
- `event.rs`: Event system for Rust→Vue communication

**Frontend (Vue.js - `src/`):**
- `bindings.ts`: Auto-generated TypeScript bindings from Rust (DO NOT EDIT)
- `composables/RunCommand.ts`: Wrapper for Tauri command invocations
- `composables/event.ts`: Event listeners for Rust→Vue messages
- `views/`: Search, Config, Manage pages

## Type-Safe Communication

### Tauri-Specta Integration
- All Rust commands use `#[tauri::command]` and `#[specta::specta]` attributes
- TypeScript bindings auto-generated in `src/bindings.ts` on debug builds
- When adding new commands:
  1. Add to `collect_commands![]` in `lib.rs`
  2. Run `bun tauri dev` to regenerate bindings
  3. Import from `bindings.ts` in Vue components

### Event System
- Rust emits events: `app_handle.emit("message", msg)` or `app_handle.emit("html", html)`
- Vue listens: `listen('message', (event) => callback(event.payload))`
- Special `html` event with `restoreHtml` callback for bidirectional communication

## Key Patterns

### Configuration Access
```rust
// Read config with minimal lock duration
let (base_url, output, template) = {
    let config = config.read();
    (config.base_url.clone(), config.output.clone(), config.template.clone())
}; // Lock released here
```

### Error Handling
- Use `anyhow::Result<T>` for flexible error propagation
- Custom error types in `error.rs` if needed
- Frontend receives `Result<T, CommandError>` via bindings

### Content Decryption
```rust
use crate::secret::decode_text;
let decoded = decode_text(&scrambled_text);
```

### Paragraph Restoration
```rust
use crate::paragraph_restorer::ParagraphRestorer;
let restorer = ParagraphRestorer::new(chapter_id);
let restored = restorer.restore(paragraphs);
```

### EPUB Building
```rust
use crate::epub_builder::{EpubBuilder, Metadata, Body, ContentBlock};
let builder = EpubBuilder::new(metadata, chapters, titles, images, exts, alts, add_catalog);
builder.build(&output_path)?;
```

## Build & Development

### Commands
```bash
# Install dependencies
bun install

# Development (GUI)
bun tauri dev

# Build for production
bun tauri build

# Rust checks
bun check      # cargo check
bun clippy     # cargo clippy

# CLI mode
cd src-tauri && cargo run --bin cli -- --book-id 1234 --volume 1
```

### Build Configuration
- Frontend: Vite + Vue 3 + UnoCSS + Naive UI
- Backend: Rust with Tokio async runtime
- Release profile: LTO enabled, stripped, panic=abort
- Dev server: Fixed port 1420 (required by Tauri)

## Project-Specific Conventions

### Chapter Naming Templates
- `{{book_title}}`: Book title
- `{{chapter_title}}`: Chapter title
- `{{chapter_number}}`: Chapter number
- `{{chapter_number:x}}`: Zero-padded to x digits

### HTTP Client Headers
- Custom headers in `client.rs::get_headers()`
- Supports both default headers and user-provided header_map
- Cookie and user_agent have fallback defaults

### File Structure
- Config: `./config.json` (auto-created with defaults)
- Index: `./index.json` (EPUB library index)
- Output: User-configurable directory

### Async Patterns
- All download operations use `async fn`
- Tokio runtime managed by Tauri
- Use `tokio::sync::broadcast` for cancellation signals

## Testing & Debugging

### CLI Testing
```bash
cargo run --bin cli -- --book-id 1234 --volume 1 --output ./test
```

### GUI Debugging
- Tauri devtools enabled in `Cargo.toml` features
- Check browser console for Vue errors
- Check terminal for Rust errors

### Common Issues
- **Port 1420 in use**: Kill process or change `vite.config.ts`
- **Binding sync issues**: Run `bun tauri dev` to regenerate
- **Config not loading**: Check `./config.json` format

## External Dependencies

- **reqwest**: HTTP client with gzip/deflate/brotli support
- **scraper**: HTML parsing
- **zip**: EPUB file creation
- **quick-xml**: EPUB metadata parsing
- **regex**: Pattern matching
- **chrono**: Date/time handling
- **image**: Image processing
- **walkdir**: Directory traversal for library management

## Adding New Features

### New Tauri Command
1. Define function in `src-tauri/src/command.rs` with `#[tauri::command]` and `#[specta::specta]`
2. Add to `collect_commands![]` in `lib.rs`
3. Run `bun tauri dev` to update `bindings.ts`
4. Use in Vue: `await commands.newCommand(params)`

### New Event
1. Emit in Rust: `app_handle.emit("event_name", payload)`
2. Listen in Vue: `listen('event_name', callback)`

### New Config Field
1. Add field to `Config` struct in `config.rs`
2. Add default function if needed
3. Update `Default` implementation
4. Frontend automatically picks up via `getConfigVue()`