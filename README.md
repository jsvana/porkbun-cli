# porkbun-cli

A command-line tool for managing DNS records and domains via the Porkbun API.

## Usage

Set your API credentials:

```bash
export PORKBUN_API_KEY="pk1_..."
export PORKBUN_SECRET_API_KEY="sk1_..."
```

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

## Building

```bash
cargo build --release
```
