use crate::CouncilSeatConfig;
use anyhow::Result;
use async_trait::async_trait;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use tracing::info;

use super::Strategy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    pub id: i64,
    pub content: String,
    pub source: String,
    pub domain: String,
    pub metadata: Option<String>,
}

pub struct LibrarianStrategy {
    config: CouncilSeatConfig,
    db: Mutex<Option<Connection>>,
    initialized: bool,
}

impl LibrarianStrategy {
    pub async fn new(config: CouncilSeatConfig) -> Result<Self> {
        Ok(Self {
            config,
            db: Mutex::new(None),
            initialized: false,
        })
    }
    
    fn init_database(&self) -> Result<Connection> {
        let db_path = &self.config.database_path;
        
        // Ensure directory exists
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS knowledge (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                source TEXT NOT NULL,
                domain TEXT NOT NULL,
                metadata TEXT,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_knowledge_domain ON knowledge(domain)",
            [],
        )?;
        
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_fts USING fts5(
                content, source, domain, content=knowledge, content_rowid=id
            )",
            [],
        )?;
        
        info!("Database initialized at {}", db_path);
        Ok(conn)
    }
    
    fn search_knowledge(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeChunk>> {
        let conn_guard = self.db.lock().unwrap();
        let conn = conn_guard.as_ref().ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;
        
        let query_pattern = format!("{}*", query);
        
        let mut stmt = conn.prepare(
            "SELECT id, content, source, domain, metadata 
             FROM knowledge 
             WHERE id IN (
                 SELECT rowid FROM knowledge_fts WHERE knowledge_fts MATCH ?1
             )
             LIMIT ?2"
        )?;
        
        let chunks = stmt.query_map([query_pattern, limit.to_string()], |row| {
            Ok(KnowledgeChunk {
                id: row.get(0)?,
                content: row.get(1)?,
                source: row.get(2)?,
                domain: row.get(3)?,
                metadata: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(chunks)
    }
    
    fn add_knowledge(&self, content: &str, source: &str, domain: &str) -> Result<()> {
        let conn_guard = self.db.lock().unwrap();
        let conn = conn_guard.as_ref().ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;
        
        conn.execute(
            "INSERT INTO knowledge (content, source, domain) VALUES (?1, ?2, ?3)",
            [content, source, domain],
        )?;
        
        // Update FTS index - get the last inserted id
        let last_id: i64 = conn.query_row("SELECT last_insert_rowid()", [], |row| row.get(0))?;
        
        conn.execute(
            "INSERT INTO knowledge_fts(rowid, content, source, domain) 
             SELECT id, content, source, domain FROM knowledge WHERE id = ?1",
            [last_id],
        )?;
        
        Ok(())
    }
}

#[async_trait]
impl Strategy for LibrarianStrategy {
    async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        info!("Initializing Librarian strategy");
        
        // Initialize database
        let conn = self.init_database()?;
        
        {
            let mut db_guard = self.db.lock().unwrap();
            *db_guard = Some(conn);
        }
        
        // Check if we have any knowledge loaded
        let count: i64 = {
            let conn_guard = self.db.lock().unwrap();
            let conn = conn_guard.as_ref().unwrap();
            conn.query_row("SELECT COUNT(*) FROM knowledge", [], |row| row.get(0))?
        };
        
        if count == 0 {
            info!("No knowledge loaded. Loading sample data...");
            self.load_sample_knowledge()?;
        }
        
        self.initialized = true;
        Ok(())
    }
    
    async fn query(&self, question: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Strategy not initialized"));
        }
        
        info!("Librarian query: {}", question);
        
        // Simple keyword extraction for search
        let keywords = extract_keywords(question);
        
        // Search for relevant knowledge
        let mut results = Vec::new();
        for keyword in keywords.iter().take(5) {
            if let Ok(chunks) = self.search_knowledge(keyword, 3) {
                results.extend(chunks);
            }
        }
        
        // Deduplicate and limit
        results.sort_by(|a, b| b.id.cmp(&a.id));
        results.dedup_by(|a, b| a.id == b.id);
        results.truncate(5);
        
        if results.is_empty() {
            return Ok("I don't have any knowledge on that topic in my library. Try loading some knowledge base documents.".to_string());
        }
        
        // Format response
        let mut response = String::from("Based on my knowledge library:\n\n");
        
        for (i, chunk) in results.iter().enumerate() {
            response.push_str(&format!("{}. {}\n", i + 1, chunk.content));
            response.push_str(&format!("   Source: {}\n\n", chunk.source));
        }
        
        response.push_str("--- End of results ---");
        
        Ok(response)
    }
    
    fn name(&self) -> &str {
        "librarian"
    }
    
    fn description(&self) -> &str {
        "The Librarian knows how to read and search static knowledge data. Ideal for answering questions about documents, manifests, and any text-based information in the system."
    }
}

impl LibrarianStrategy {
    fn load_sample_knowledge(&self) -> Result<()> {
        // Sample knowledge for demonstration
        let sample_knowledge = vec![
            ("Survon is a smart homestead system that manages devices, monitors health, and provides intelligent automation for off-grid living.", "README.md", "general"),
            ("The runtime-base-rust project provides the core Survon TUI for managing modules and viewing system status.", "docs/runtime.md", "software"),
            ("BLE field units communicate via Bluetooth Low Energy to send telemetry data to the central system.", "docs/ble.md", "hardware"),
            ("The council system provides multi-advisor consensus for complex decisions.", "docs/council.md", "software"),
            ("Raspberry Pi 3B v1.2 is the recommended hardware for running Survon OS.", "docs/hardware.md", "hardware"),
            ("Knowledge can be loaded into the librarian via manifest files in the manifests/ directory.", "docs/knowledge.md", "software"),
            ("The Overseer module manages device discovery, trust, and configuration.", "docs/overseer.md", "software"),
            ("The valve_control module manages water and gas valves in the homestead.", "docs/valve_control.md", "hardware"),
        ];
        
        for (content, source, domain) in &sample_knowledge {
            self.add_knowledge(content, source, domain)?;
        }
        
        info!("Loaded {} sample knowledge items", sample_knowledge.len());
        Ok(())
    }
}

fn extract_keywords(text: &str) -> Vec<String> {
    let text = text.to_lowercase();
    let stop_words = ["what", "is", "the", "a", "an", "how", "do", "does", "can", "could", "would", "should", "i", "me", "my", "we", "our", "you", "your", "it", "its", "to", "of", "in", "for", "on", "with", "at", "by", "from", "or", "and", "be", "are", "was", "were", "this", "that", "these", "those"];
    
    text.split_whitespace()
        .filter(|word| !stop_words.contains(word))
        .map(|s| s.to_string())
        .collect()
}
