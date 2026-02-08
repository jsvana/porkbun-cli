use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.porkbun.com/api/json/v3";

#[derive(Deserialize)]
struct Config {
    api_key: String,
    secret_key: String,
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("porkbun-cli")
        .join("config.toml")
}

fn load_config() -> Result<Config> {
    let path = config_path();
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file at {}", path.display()))?;
    toml::from_str(&contents).with_context(|| format!("Failed to parse {}", path.display()))
}

#[derive(Parser)]
#[command(name = "porkbun", about = "CLI for managing Porkbun DNS records")]
struct Cli {
    /// Print column headers before list output
    #[arg(long)]
    headers: bool,

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

    let config = load_config()?;
    let auth = Auth {
        apikey: config.api_key,
        secretapikey: config.secret_key,
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

            if cli.headers {
                println!("DOMAIN\tSTATUS\tCREATED\tEXPIRES");
            }
            for d in resp.domains {
                println!("{}\t{}\t{}\t{}", d.domain, d.status, d.create_date, d.expire_date);
            }
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

                if cli.headers {
                    println!("ID\tNAME\tTYPE\tCONTENT\tTTL");
                }
                for r in resp.records {
                    println!(
                        "{}\t{}\t{}\t{}\t{}",
                        r.id, r.name, r.record_type, r.content, r.ttl,
                    );
                }
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

