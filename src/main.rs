use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

const BASE_URL: &str = "https://api.porkbun.com/api/json/v3";

#[derive(Parser)]
#[command(name = "porkbun", about = "CLI for managing Porkbun DNS records")]
struct Cli {
    /// Porkbun API key (or set PORKBUN_API_KEY)
    #[arg(long, env = "PORKBUN_API_KEY")]
    api_key: String,

    /// Porkbun secret API key (or set PORKBUN_SECRET_API_KEY)
    #[arg(long, env = "PORKBUN_SECRET_API_KEY")]
    secret_key: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List all domains
    Domains,

    /// Manage DNS records
    Dns {
        #[command(subcommand)]
        action: DnsAction,
    },
}

#[derive(Subcommand)]
enum DnsAction {
    /// List DNS records for a domain
    List {
        /// Domain name
        domain: String,
    },

    /// Create a DNS record
    Create {
        /// Domain name
        domain: String,
        /// Record type (A, AAAA, CNAME, MX, TXT, NS, SRV, CAA, etc.)
        #[arg(short = 't', long = "type")]
        record_type: String,
        /// Record content/value
        content: String,
        /// Subdomain (omit for root)
        #[arg(short, long)]
        name: Option<String>,
        /// TTL in seconds (min 600)
        #[arg(long)]
        ttl: Option<String>,
        /// Priority (for MX, SRV, etc.)
        #[arg(short, long)]
        prio: Option<String>,
    },

    /// Edit a DNS record by ID
    Edit {
        /// Domain name
        domain: String,
        /// Record ID
        id: String,
        /// Record type
        #[arg(short = 't', long = "type")]
        record_type: String,
        /// Record content/value
        content: String,
        /// Subdomain (omit for root)
        #[arg(short, long)]
        name: Option<String>,
        /// TTL in seconds (min 600)
        #[arg(long)]
        ttl: Option<String>,
        /// Priority
        #[arg(short, long)]
        prio: Option<String>,
    },

    /// Delete a DNS record by ID
    Delete {
        /// Domain name
        domain: String,
        /// Record ID
        id: String,
    },

    /// Delete DNS records by name and type
    DeleteByNameType {
        /// Domain name
        domain: String,
        /// Record type
        #[arg(short = 't', long = "type")]
        record_type: String,
        /// Subdomain (omit for root)
        #[arg(short, long)]
        name: Option<String>,
    },
}

// --- API types ---

#[derive(Serialize)]
struct Auth {
    apikey: String,
    secretapikey: String,
}

#[derive(Serialize)]
struct CreateEditRecord {
    #[serde(flatten)]
    auth: Auth,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(rename = "type")]
    record_type: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prio: Option<String>,
}

#[derive(Deserialize)]
struct ApiResponse {
    status: String,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Deserialize)]
struct DomainListResponse {
    status: String,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    domains: Vec<DomainInfo>,
}

#[derive(Deserialize)]
struct DomainInfo {
    domain: String,
    status: String,
    #[serde(default, rename = "expireDate")]
    expire_date: String,
    #[serde(default, rename = "createDate")]
    create_date: String,
}

#[derive(Tabled)]
struct DomainRow {
    #[tabled(rename = "Domain")]
    domain: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Created")]
    created: String,
    #[tabled(rename = "Expires")]
    expires: String,
}

#[derive(Deserialize)]
struct DnsListResponse {
    status: String,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    records: Vec<DnsRecord>,
}

#[derive(Deserialize)]
struct DnsRecord {
    id: String,
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    content: String,
    ttl: String,
    #[serde(default)]
    prio: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Tabled)]
struct DnsRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    record_type: String,
    #[tabled(rename = "Content")]
    content: String,
    #[tabled(rename = "TTL")]
    ttl: String,
    #[tabled(rename = "Prio")]
    prio: String,
    #[tabled(rename = "Notes")]
    notes: String,
}

#[derive(Deserialize)]
struct CreateResponse {
    status: String,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    id: Option<serde_json::Value>,
}

fn check_status(status: &str, message: &Option<String>) -> Result<()> {
    if status != "SUCCESS" {
        bail!(
            "API error: {}",
            message.as_deref().unwrap_or("unknown error")
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();
    let auth = Auth {
        apikey: cli.api_key.clone(),
        secretapikey: cli.secret_key.clone(),
    };

    match cli.command {
        Command::Domains => {
            let resp: DomainListResponse = client
                .post(format!("{BASE_URL}/domain/listAll"))
                .json(&auth)
                .send()
                .await
                .context("Failed to contact Porkbun API")?
                .json()
                .await?;

            check_status(&resp.status, &resp.message)?;

            if resp.domains.is_empty() {
                println!("No domains found.");
                return Ok(());
            }

            let rows: Vec<DomainRow> = resp
                .domains
                .into_iter()
                .map(|d| DomainRow {
                    domain: d.domain,
                    status: d.status,
                    created: d.create_date,
                    expires: d.expire_date,
                })
                .collect();

            println!("{}", Table::new(rows));
        }

        Command::Dns { action } => match action {
            DnsAction::List { domain } => {
                let resp: DnsListResponse = client
                    .post(format!("{BASE_URL}/dns/retrieve/{domain}"))
                    .json(&auth)
                    .send()
                    .await
                    .context("Failed to contact Porkbun API")?
                    .json()
                    .await?;

                check_status(&resp.status, &resp.message)?;

                if resp.records.is_empty() {
                    println!("No DNS records found for {domain}.");
                    return Ok(());
                }

                let rows: Vec<DnsRow> = resp.records.into_iter().map(dns_row).collect();
                println!("{}", Table::new(rows));
            }

            DnsAction::Create {
                domain,
                record_type,
                content,
                name,
                ttl,
                prio,
            } => {
                let body = CreateEditRecord {
                    auth,
                    name: name.clone(),
                    record_type: record_type.clone(),
                    content: content.clone(),
                    ttl,
                    prio,
                };

                let resp: CreateResponse = client
                    .post(format!("{BASE_URL}/dns/create/{domain}"))
                    .json(&body)
                    .send()
                    .await
                    .context("Failed to contact Porkbun API")?
                    .json()
                    .await?;

                check_status(&resp.status, &resp.message)?;

                let id_str = resp
                    .id
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".into());
                let display_name = name.as_deref().unwrap_or("(root)");
                println!(
                    "Created {record_type} record for {display_name}.{domain} -> {content} (id: {id_str})"
                );
            }

            DnsAction::Edit {
                domain,
                id,
                record_type,
                content,
                name,
                ttl,
                prio,
            } => {
                let body = CreateEditRecord {
                    auth,
                    name: name.clone(),
                    record_type: record_type.clone(),
                    content: content.clone(),
                    ttl,
                    prio,
                };

                let resp: ApiResponse = client
                    .post(format!("{BASE_URL}/dns/edit/{domain}/{id}"))
                    .json(&body)
                    .send()
                    .await
                    .context("Failed to contact Porkbun API")?
                    .json()
                    .await?;

                check_status(&resp.status, &resp.message)?;
                println!("Updated record {id} on {domain}.");
            }

            DnsAction::Delete { domain, id } => {
                let resp: ApiResponse = client
                    .post(format!("{BASE_URL}/dns/delete/{domain}/{id}"))
                    .json(&auth)
                    .send()
                    .await
                    .context("Failed to contact Porkbun API")?
                    .json()
                    .await?;

                check_status(&resp.status, &resp.message)?;
                println!("Deleted record {id} from {domain}.");
            }

            DnsAction::DeleteByNameType {
                domain,
                record_type,
                name,
            } => {
                let subdomain = name.as_deref().unwrap_or("");
                let url = if subdomain.is_empty() {
                    format!("{BASE_URL}/dns/deleteByNameType/{domain}/{record_type}")
                } else {
                    format!("{BASE_URL}/dns/deleteByNameType/{domain}/{record_type}/{subdomain}")
                };

                let resp: ApiResponse = client
                    .post(&url)
                    .json(&auth)
                    .send()
                    .await
                    .context("Failed to contact Porkbun API")?
                    .json()
                    .await?;

                check_status(&resp.status, &resp.message)?;
                let display = if subdomain.is_empty() {
                    format!("root {record_type}")
                } else {
                    format!("{subdomain} {record_type}")
                };
                println!("Deleted {display} records from {domain}.");
            }
        },
    }

    Ok(())
}

fn dns_row(r: DnsRecord) -> DnsRow {
    DnsRow {
        id: r.id,
        name: r.name,
        record_type: r.record_type,
        content: r.content,
        ttl: r.ttl,
        prio: r.prio.unwrap_or_default(),
        notes: r.notes.unwrap_or_default(),
    }
}
