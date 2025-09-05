use crate::Decision;
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

/// Logs AI decisions for debugging and analysis
pub struct DecisionLogger {
    enabled: bool,
    log_file: Option<File>,
    log_buffer: Vec<LogEntry>,
    max_buffer_size: usize,
}

#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    ai_id: Uuid,
    tick: u64,
    decision: DecisionSummary,
}

#[derive(Debug, Clone)]
struct DecisionSummary {
    action: String,
    confidence: f32,
    reasoning: String,
    hat: String,
    messages_sent: usize,
}

impl DecisionLogger {
    pub fn new(enabled: bool) -> Self {
        let log_file = if enabled {
            // Use a daily log file instead of per-second
            let today = Utc::now().format("%Y%m%d");
            let filename = format!("ai_decisions_{today}.log");
            let path = PathBuf::from("logs").join(filename);

            // Create logs directory if it doesn't exist
            std::fs::create_dir_all("logs").ok();

            // Use append mode to keep writing to the same file
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .ok()
        } else {
            None
        };

        Self {
            enabled,
            log_file,
            log_buffer: Vec::new(),
            max_buffer_size: 1000,
        }
    }

    /// Log a decision
    pub fn log_decision(&mut self, ai_id: Uuid, decision: &Decision) {
        if !self.enabled {
            return;
        }

        let summary = DecisionSummary {
            action: decision
                .chosen_action
                .as_ref()
                .map(|a| format!("{a:?}"))
                .unwrap_or_else(|| "None".to_string()),
            confidence: decision.confidence,
            reasoning: decision.reasoning.clone(),
            hat: "Unknown".to_string(), // Would need to pass this in
            messages_sent: decision.messages.len(),
        };

        let entry = LogEntry {
            timestamp: Utc::now(),
            ai_id,
            tick: 0, // Would need to pass game tick
            decision: summary,
        };

        // Format the entry before borrowing the file
        let formatted_entry = self.format_entry(&entry);

        // Write to file immediately if available
        if let Some(ref mut file) = self.log_file {
            writeln!(file, "{formatted_entry}").ok();
        }

        // Also keep in memory buffer
        self.log_buffer.push(entry);

        // Trim buffer if too large
        if self.log_buffer.len() > self.max_buffer_size {
            self.log_buffer.remove(0);
        }
    }

    /// Get recent decisions for an AI
    pub fn get_recent_decisions(&self, ai_id: Uuid, count: usize) -> Vec<String> {
        self.log_buffer
            .iter()
            .rev()
            .filter(|entry| entry.ai_id == ai_id)
            .take(count)
            .map(|entry| self.format_entry(entry))
            .collect()
    }

    /// Format a log entry
    fn format_entry(&self, entry: &LogEntry) -> String {
        format!(
            "[{}] AI {} | Action: {} (conf: {:.2}) | Reason: {} | Messages: {}",
            entry.timestamp.format("%H:%M:%S%.3f"),
            &entry.ai_id.to_string()[..8],
            entry.decision.action,
            entry.decision.confidence,
            entry.decision.reasoning,
            entry.decision.messages_sent,
        )
    }

    /// Export logs to JSON
    pub fn export_json(&self, path: PathBuf) -> std::io::Result<()> {
        let json_entries: Vec<serde_json::Value> = self
            .log_buffer
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "timestamp": entry.timestamp.to_rfc3339(),
                    "ai_id": entry.ai_id.to_string(),
                    "tick": entry.tick,
                    "action": entry.decision.action,
                    "confidence": entry.decision.confidence,
                    "reasoning": entry.decision.reasoning,
                    "hat": entry.decision.hat,
                    "messages_sent": entry.decision.messages_sent,
                })
            })
            .collect();

        let json = serde_json::json!({
            "version": "1.0",
            "entries": json_entries,
        });

        let mut file = File::create(path)?;
        file.write_all(serde_json::to_string_pretty(&json)?.as_bytes())?;
        Ok(())
    }
}

/// Performance metrics for AI system
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AIMetrics {
    pub total_decisions: u64,
    pub average_decision_time_ms: f32,
    pub decisions_per_second: f32,
    pub message_count: u64,
    pub task_success_rate: f32,
}

impl AIMetrics {
    pub fn new() -> Self {
        Self {
            total_decisions: 0,
            average_decision_time_ms: 0.0,
            decisions_per_second: 0.0,
            message_count: 0,
            task_success_rate: 0.0,
        }
    }

    /// Update metrics with new decision timing
    pub fn record_decision(&mut self, decision_time_ms: f32, message_count: usize) {
        self.total_decisions += 1;
        self.message_count += message_count as u64;

        // Update rolling average
        let alpha = 0.1; // Smoothing factor
        self.average_decision_time_ms =
            self.average_decision_time_ms * (1.0 - alpha) + decision_time_ms * alpha;
    }
}

/// Debug visualization data for egui client
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AIVisualizationData {
    pub ai_states: Vec<AIStateSnapshot>,
    pub communication_graph: CommunicationGraph,
    pub decision_timeline: Vec<DecisionEvent>,
    pub performance_metrics: AIMetrics,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AIStateSnapshot {
    pub ai_id: Uuid,
    pub position: (f32, f32),
    pub current_hat: String,
    pub current_action: String,
    pub health_status: String,
    pub confidence: f32,
    pub known_threats: Vec<ThreatInfo>,
    pub known_opportunities: Vec<OpportunityInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThreatInfo {
    pub threat_type: String,
    pub position: (f32, f32),
    pub severity: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpportunityInfo {
    pub opportunity_type: String,
    pub position: (f32, f32),
    pub value: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommunicationGraph {
    pub nodes: Vec<CommNode>,
    pub edges: Vec<CommEdge>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommNode {
    pub ai_id: Uuid,
    pub is_captain: bool,
    pub message_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub message_count: u32,
    pub last_message_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionEvent {
    pub timestamp: String,
    pub ai_id: Uuid,
    pub decision_type: String,
    pub confidence: f32,
}

impl Default for AIMetrics {
    fn default() -> Self {
        Self::new()
    }
}
