use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod strategies;
use strategies::{librarian::LibrarianStrategy, Strategy};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilSeatConfig {
    pub strategy: String,
    pub strategy_config: serde_json::Value,
    pub llm_endpoint: Option<String>,
    pub llm_api_key: Option<String>,
    pub database_path: String,
    pub log_level: String,
}

impl Default for CouncilSeatConfig {
    fn default() -> Self {
        Self {
            strategy: env::var("COUNCIL_STRATEGY").unwrap_or_else(|_| "librarian".to_string()),
            strategy_config: serde_json::json!({}),
            llm_endpoint: env::var("LLM_ENDPOINT").ok(),
            llm_api_key: env::var("LLM_API_KEY").ok(),
            database_path: env::var("DATABASE_PATH").unwrap_or_else(|_| "./data/council.db".to_string()),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        }
    }
}

pub struct CouncilSeat {
    config: CouncilSeatConfig,
    strategy: Arc<RwLock<Box<dyn Strategy + Send + Sync>>>,
}

impl CouncilSeat {
    pub async fn new() -> Result<Self> {
        let config = CouncilSeatConfig::default();
        
        info!("Initializing Council Seat with strategy: {}", config.strategy);
        
        let strategy = Self::create_strategy(&config).await?;
        
        Ok(Self {
            config,
            strategy: Arc::new(RwLock::new(strategy)),
        })
    }
    
    async fn create_strategy(config: &CouncilSeatConfig) -> Result<Box<dyn Strategy + Send + Sync>> {
        match config.strategy.to_lowercase().as_str() {
            "librarian" => {
                info!("Creating Librarian strategy");
                let strategy = LibrarianStrategy::new(config.clone()).await?;
                Ok(Box::new(strategy))
            }
            "medicine" | "doctor" => {
                info!("Creating Medicine strategy");
                // Placeholder - would use PubMed-trained LLM
                Err(anyhow::anyhow!("Medicine strategy not yet implemented"))
            }
            "mechanical" => {
                info!("Creating Mechanical strategy");
                Err(anyhow::anyhow!("Mechanical strategy not yet implemented"))
            }
            "botany" => {
                info!("Creating Botany strategy");
                Err(anyhow::anyhow!("Botany strategy not yet implemented"))
            }
            "veterinary" => {
                info!("Creating Veterinary strategy");
                Err(anyhow::anyhow!("Veterinary strategy not yet implemented"))
            }
            "building" => {
                info!("Creating Building strategy");
                Err(anyhow::anyhow!("Building strategy not yet implemented"))
            }
            "survival" => {
                info!("Creating Survival strategy");
                Err(anyhow::anyhow!("Survival strategy not yet implemented"))
            }
            s => {
                warn!("Unknown strategy: {}, defaulting to librarian", s);
                let strategy = LibrarianStrategy::new(config.clone()).await?;
                Ok(Box::new(strategy))
            }
        }
    }
    
    pub async fn query(&self, question: &str) -> Result<String> {
        info!("Processing query: {}", question);
        let strategy = self.strategy.read().await;
        strategy.query(question).await
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("Council Seat started with strategy: {}", self.config.strategy);
        
        let mut strategy = self.strategy.write().await;
        strategy.initialize().await?;
        
        Ok(())
    }
}

fn setup_logging(log_level: &str) {
    let log_dir = std::path::Path::new("./logs");
    std::fs::create_dir_all(log_dir).ok();
    
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        log_dir,
        "council-seat.log",
    );
    
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(non_blocking))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();
    
    // Store guard to keep logging alive
    std::mem::forget(_guard);
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = CouncilSeatConfig::default();
    setup_logging(&config.log_level);
    
    info!("Survon Council Seat v0.1.0");
    info!("Strategy: {}", config.strategy);
    
    color_eyre::config::HookBuilder::new()
        .install()
        .ok();
    
    let seat = CouncilSeat::new().await?;
    seat.start().await?;
    
    info!("Council Seat ready. Waiting for queries...");
    
    // Demo: answer a sample question
    let question = "What is the purpose of the survon system?";
    match seat.query(question).await {
        Ok(response) => {
            println!("\nQuestion: {}", question);
            println!("Answer: {}\n", response);
        }
        Err(e) => {
            error!("Query failed: {}", e);
        }
    }
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    
    info!("Council Seat shutting down");
    Ok(())
}
