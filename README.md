# porkbun-cli

A command-line tool for managing DNS records and domains via the Porkbun API.

## Configuration

Create `~/.config/porkbun-cli/config.toml` with your API credentials:

```toml
api_key = "pk1_..."
secret_key = "sk1_..."
```

Each domain must have API access enabled in the [Porkbun dashboard](https://porkbun.com/account/domainsSpe498702).

## Usage

List domains:

```bash
porkbun-cli domains
```

List DNS records:

```bash
porkbun-cli dns list example.com
```

Create a record:

```bash
porkbun-cli dns create example.com -t A 1.2.3.4 --name www
```

Edit a record by ID:

```bash
porkbun-cli dns edit example.com 123456789 -t A 5.6.7.8 --name www
```

Delete a record:

```bash
porkbun-cli dns delete example.com 123456789
```

Use `--headers` before the subcommand to print column headers:

```bash
porkbun-cli --headers dns list example.com
```

## Building

```bash
cargo build --release
```

## Claude Code Skill

This repo includes a [Claude Code](https://claude.ai/code) skill at `.claude/skills/porkbun-dns/` that teaches Claude how to use the CLI for DNS management. It is automatically available when working in this repo.

To install it globally (so Claude can use it from any project):

```
/plugin marketplace add jsvana/porkbun-cli
/plugin install porkbun-dns@jsvana/porkbun-cli
```
