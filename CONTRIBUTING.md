# Contributing to Birdhouse Camera

## Development Setup

1. Install the ESP Rust toolchain:

   ```bash
   make setup
   . $HOME/export-esp.sh
   ```

2. Install pre-commit hooks:

   ```bash
   make hooks
   ```

## Before Committing

Run the full CI check suite locally:

```bash
make ci
```

This runs:
- `cargo fmt -- --check` (formatting)
- `cargo clippy` with pedantic lints (linting)
- `cargo build` (compilation)

Pre-commit hooks will also run these checks automatically on `git commit`.

## Code Style

- Follow the existing code conventions in the project
- Use `rustfmt` for formatting (configured in `rustfmt.toml`)
- All clippy pedantic warnings are enabled — fix them rather than suppressing
- Keep `unsafe` blocks minimal and well-documented
- Use `log::info!`, `log::warn!`, etc. for logging (not `println!`)

## Adding New Modules

1. Create the module file in `src/`
2. Add `mod your_module;` to `src/main.rs`
3. Follow the existing documentation style with module-level doc comments

## Testing

Since this is an embedded project, most testing requires physical hardware.
When adding new logic that can be tested independently (e.g., detection
algorithms, MQTT payload formatting), consider extracting it into
hardware-independent functions that could be unit tested on the host.
