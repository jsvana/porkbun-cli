# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test

```bash
cargo build              # Debug build
cargo build --release    # Release build (installed to ~/bin/porkbun-cli)
cargo fmt --check        # Check formatting
cargo clippy -- -D warnings  # Lint
cargo test               # Run tests (none currently)
```

## What This Is

A CLI for the Porkbun DNS API (https://porkbun.com/api/json/v3/documentation). Single-binary, single-file Rust project.

**Auth:** Reads `api_key` and `secret_key` from `~/.config/porkbun-cli/config.toml`.

## Architecture

Everything lives in `src/main.rs`. The structure is:

- **CLI layer** — clap derive structs (`Cli`, `Command`, `DnsAction`) define the command tree: `domains`, `dns {list,create,edit,delete,delete-by-name-type}`
- **API types** — serde structs for request/response bodies. `Auth` is flattened into request bodies via `#[serde(flatten)]`. All Porkbun endpoints are JSON POST.
- **Display types** — `DomainRow`/`DnsRow` with `tabled::Tabled` derive for table output
- **`main()`** — matches on the command enum, makes the reqwest call, checks status, prints output

All API calls follow the same pattern: POST to `BASE_URL/<endpoint>` with auth JSON body, deserialize response, call `check_status()`, print result.
