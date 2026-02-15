---
name: porkbun-dns
description: Use when managing DNS records on Porkbun, listing domains, creating/editing/deleting DNS entries, or troubleshooting domain configuration
---

# Porkbun DNS Management

## Overview

Manages DNS records and domains on Porkbun via the `porkbun-cli` tool.

**Requires:** Config file at `~/.config/porkbun-cli/config.toml` with `api_key` and `secret_key`.

**Important:** Each domain must have API access enabled in the Porkbun dashboard before it can be managed via this CLI. Domains without API access opted in will return auth errors.

Source code is in this repository.

## Output Format

Output is tab-separated, one record per line, for easy use with `grep`, `cut`, `awk`, etc. Use `--headers` to print a header row.

## CLI Reference

### List all domains

```bash
porkbun-cli domains
porkbun-cli --headers domains   # with column headers
```

### List DNS records for a domain

```bash
porkbun-cli dns list example.com
porkbun-cli --headers dns list example.com   # with column headers
```

### Create a DNS record

```bash
# A record for subdomain
porkbun-cli dns create example.com -t A 1.2.3.4 --name www

# A record at root
porkbun-cli dns create example.com -t A 1.2.3.4

# CNAME
porkbun-cli dns create example.com -t CNAME target.example.com --name blog

# MX with priority
porkbun-cli dns create example.com -t MX mail.example.com -p 10

# TXT record (e.g., SPF, DKIM, verification)
porkbun-cli dns create example.com -t TXT "v=spf1 include:_spf.google.com ~all"

# With custom TTL
porkbun-cli dns create example.com -t A 1.2.3.4 --name www --ttl 3600
```

### Edit a DNS record by ID

```bash
# Get the record ID from `dns list`, then edit
porkbun-cli dns edit example.com 123456789 -t A 5.6.7.8 --name www
```

### Delete a DNS record by ID

```bash
porkbun-cli dns delete example.com 123456789
```

### Delete DNS records by name and type

```bash
# Delete all A records for a subdomain
porkbun-cli dns delete-by-name-type example.com -t A --name www

# Delete all root TXT records
porkbun-cli dns delete-by-name-type example.com -t TXT
```

## Supported Record Types

A, AAAA, CNAME, MX, TXT, NS, SRV, TLSA, CAA, HTTPS, SVCB, SSHFP

## Workflow: Point a Subdomain to an IP

1. List current records: `porkbun-cli dns list example.com`
2. Check if a conflicting record exists (same name/type)
3. If exists: delete it first or edit by ID
4. Create: `porkbun-cli dns create example.com -t A 1.2.3.4 --name sub`
5. Verify: `porkbun-cli dns list example.com`

## Workflow: Set Up Email (MX Records)

1. Delete any existing MX records: `porkbun-cli dns delete-by-name-type example.com -t MX`
2. Create MX records with priorities:
   ```bash
   porkbun-cli dns create example.com -t MX aspmx.l.google.com -p 1
   porkbun-cli dns create example.com -t MX alt1.aspmx.l.google.com -p 5
   ```
3. Add SPF TXT record:
   ```bash
   porkbun-cli dns create example.com -t TXT "v=spf1 include:_spf.google.com ~all"
   ```

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Forgetting `--name` creates a root record | Always specify `--name` for subdomains |
| Duplicate records | List first, delete conflicts before creating |
| TTL too low | Minimum is 600 seconds |
| Missing config | Create `~/.config/porkbun-cli/config.toml` with `api_key` and `secret_key` |
| API access not enabled | Enable API access per-domain in Porkbun dashboard |

## Confirming Changes

**Always confirm with the user before creating, editing, or deleting DNS records.** DNS changes propagate globally and can cause downtime. Show the planned action and get explicit approval.

## Troubleshooting

- **"API error: invalid api key"** - Check config file credentials
- **"API error: ... not auth"** - API access must be enabled for that domain in Porkbun settings
- **Record not appearing** - DNS propagation can take minutes to hours; verify with `dns list` first
