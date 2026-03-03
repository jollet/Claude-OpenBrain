use anyhow::Result;
use clap::{Parser, Subcommand};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "brain", about = "Open Brain CLI")]
struct Cli {
    /// Base URL of the brain-core API
    #[arg(long, env = "BRAIN_API_URL", default_value = "http://localhost:3000")]
    api_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new thought
    Add {
        /// The content of the thought
        content: String,
        /// Optional tags (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// List all thoughts
    List {
        /// Number of results
        #[arg(short, long, default_value = "20")]
        limit: i64,
        /// Offset for pagination
        #[arg(short, long, default_value = "0")]
        offset: i64,
    },
    /// Get a thought by ID
    Get {
        /// Thought ID
        id: i64,
    },
    /// Delete a thought by ID
    Delete {
        /// Thought ID
        id: i64,
    },
    /// Show server health
    Health,
}

#[derive(Debug, Serialize)]
struct CreateRequest {
    content: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Thought {
    id: i64,
    content: String,
    tags: Vec<String>,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    db_healthy: bool,
    embedding_backend: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = Client::new();

    match cli.command {
        Commands::Add { content, tags } => {
            let resp = client
                .post(format!("{}/api/thoughts", cli.api_url))
                .json(&CreateRequest { content, tags })
                .send()?;
            if resp.status().is_success() {
                let thought: Thought = resp.json()?;
                println!("✓ Created thought #{}", thought.id);
                println!("  Content: {}", thought.content);
                if !thought.tags.is_empty() {
                    println!("  Tags: {}", thought.tags.join(", "));
                }
            } else {
                eprintln!("✗ Error: {}", resp.status());
            }
        }
        Commands::List { limit, offset } => {
            let resp = client
                .get(format!("{}/api/thoughts", cli.api_url))
                .query(&[("limit", limit), ("offset", offset)])
                .send()?;
            let thoughts: Vec<Thought> = resp.json()?;
            if thoughts.is_empty() {
                println!("No thoughts yet.");
            } else {
                for t in &thoughts {
                    let tags = if t.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", t.tags.join(", "))
                    };
                    println!("#{} {}{}", t.id, t.content, tags);
                }
                println!("\n{} thought(s)", thoughts.len());
            }
        }
        Commands::Get { id } => {
            let resp = client
                .get(format!("{}/api/thoughts/{}", cli.api_url, id))
                .send()?;
            if resp.status().is_success() {
                let t: Thought = resp.json()?;
                println!("#{} — {}", t.id, t.created_at);
                println!("{}", t.content);
                if !t.tags.is_empty() {
                    println!("Tags: {}", t.tags.join(", "));
                }
            } else {
                eprintln!("Thought #{} not found.", id);
            }
        }
        Commands::Delete { id } => {
            let resp = client
                .delete(format!("{}/api/thoughts/{}", cli.api_url, id))
                .send()?;
            if resp.status().is_success() {
                println!("✓ Thought #{} deleted.", id);
            } else {
                eprintln!("✗ Thought #{} not found.", id);
            }
        }
        Commands::Health => {
            let resp = client
                .get(format!("{}/health", cli.api_url))
                .send()?;
            let h: HealthResponse = resp.json()?;
            println!("Status:    {}", h.status);
            println!("Version:   {}", h.version);
            println!("DB:        {}", if h.db_healthy { "healthy" } else { "unhealthy" });
            println!("Embedding: {}", h.embedding_backend);
        }
    }
    Ok(())
}
