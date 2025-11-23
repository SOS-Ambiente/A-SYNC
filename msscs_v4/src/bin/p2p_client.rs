// P2P CLIENT - Command-line client for P2P storage
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::json;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "p2p-client")]
#[command(about = "MSSCS P2P Storage Client", long_about = None)]
struct Args {
    /// Server URL
    #[arg(short, long, default_value = "http://localhost:8080")]
    server: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Upload a file
    Upload {
        /// Local file path
        file: PathBuf,
        
        /// Remote path
        #[arg(short, long)]
        path: String,
    },
    
    /// Download a file
    Download {
        /// Remote path
        path: String,
        
        /// Local output file
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Delete a file
    Delete {
        /// Remote path
        path: String,
    },
    
    /// List files
    List,
    
    /// Show statistics
    Stats,
    
    /// Health check
    Health,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = Client::new();
    
    match args.command {
        Commands::Upload { file, path } => {
            println!("📤 Uploading {} to {}", file.display(), path);
            
            use base64::{Engine as _, engine::general_purpose};
            
            let data = std::fs::read(&file)?;
            let encoded = general_purpose::STANDARD.encode(&data);
            
            let response = client
                .post(format!("{}/upload", args.server))
                .json(&json!({
                    "path": path,
                    "content": encoded
                }))
                .send()
                .await?;
            
            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                println!("✅ Upload successful!");
                println!("   UUID: {}", result["uuid"]);
            } else {
                println!("❌ Upload failed: {}", response.status());
            }
        }
        
        Commands::Download { path, output } => {
            println!("📥 Downloading {} to {}", path, output.display());
            
            let response = client
                .get(format!("{}/download/{}", args.server, path))
                .send()
                .await?;
            
            if response.status().is_success() {
                use base64::{Engine as _, engine::general_purpose};
                
                let result: serde_json::Value = response.json().await?;
                let content = result["content"].as_str().unwrap();
                let decoded = general_purpose::STANDARD.decode(content)?;
                
                std::fs::write(&output, decoded)?;
                println!("✅ Download successful!");
            } else {
                println!("❌ Download failed: {}", response.status());
            }
        }
        
        Commands::Delete { path } => {
            println!("🗑️  Deleting {}", path);
            
            let response = client
                .delete(format!("{}/delete/{}", args.server, path))
                .send()
                .await?;
            
            if response.status().is_success() {
                println!("✅ Delete successful!");
            } else {
                println!("❌ Delete failed: {}", response.status());
            }
        }
        
        Commands::List => {
            println!("📋 Listing files...");
            
            let response = client
                .get(format!("{}/files", args.server))
                .send()
                .await?;
            
            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                let files = result["files"].as_array().unwrap();
                
                if files.is_empty() {
                    println!("   No files found");
                } else {
                    for file in files {
                        println!("   - {}", file.as_str().unwrap());
                    }
                }
            } else {
                println!("❌ List failed: {}", response.status());
            }
        }
        
        Commands::Stats => {
            println!("📊 Fetching statistics...");
            
            let response = client
                .get(format!("{}/stats", args.server))
                .send()
                .await?;
            
            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                println!("   Total files: {}", result["total_files"]);
                println!("   Cached blocks: {}", result["cached_blocks"]);
                println!("   Connected peers: {}", result["connected_peers"]);
            } else {
                println!("❌ Stats failed: {}", response.status());
            }
        }
        
        Commands::Health => {
            println!("🏥 Checking health...");
            
            let response = client
                .get(format!("{}/health", args.server))
                .send()
                .await?;
            
            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                println!("   Status: {}", result["status"]);
                println!("   Version: {}", result["version"]);
            } else {
                println!("❌ Health check failed: {}", response.status());
            }
        }
    }
    
    Ok(())
}
