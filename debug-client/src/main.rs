use eframe::egui;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use ai::{AIVisualizationData, AIDebugInfo, AIMetrics};
use shared::*;

mod network;
use network::DebugConnection;

/// Main application state
struct AIDebugApp {
    /// Connection to game server
    connection: Arc<Mutex<Option<DebugConnection>>>,
    /// Current game state
    game_state: GameState,
    /// AI visualization data
    ai_data: HashMap<Uuid, AIVisualizationData>,
    /// Selected AI for detailed view
    selected_ai: Option<Uuid>,
    /// Simulation controls
    sim_paused: bool,
    sim_speed: f32,
    /// UI state
    show_communication_graph: bool,
    show_decision_timeline: bool,
    show_performance_metrics: bool,
    /// Server address
    server_address: String,
    connection_status: ConnectionStatus,
}

#[derive(Debug, Clone)]
enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone, Default)]
struct GameState {
    players: HashMap<Uuid, PlayerState>,
    mechs: HashMap<Uuid, MechState>,
    resources: HashMap<Uuid, ResourceState>,
    projectiles: Vec<ProjectileState>,
}

impl Default for AIDebugApp {
    fn default() -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
            game_state: GameState::default(),
            ai_data: HashMap::new(),
            selected_ai: None,
            sim_paused: false,
            sim_speed: 1.0,
            show_communication_graph: true,
            show_decision_timeline: true,
            show_performance_metrics: true,
            server_address: "ws://127.0.0.1:14191/debug".to_string(),
            connection_status: ConnectionStatus::Disconnected,
        }
    }
}

impl AIDebugApp {
    fn connect_to_server(&mut self) {
        self.connection_status = ConnectionStatus::Connecting;
        
        let address = self.server_address.clone();
        let connection = self.connection.clone();
        
        // Spawn connection task
        std::thread::spawn(move || {
            match DebugConnection::connect(&address) {
                Ok(conn) => {
                    *connection.lock().unwrap() = Some(conn);
                }
                Err(e) => {
                    log::error!("Failed to connect: {}", e);
                }
            }
        });
    }
    
    fn update_from_server(&mut self) {
        let mut messages = Vec::new();
        let mut is_connected = false;
        
        if let Ok(conn_guard) = self.connection.lock() {
            if let Some(conn) = conn_guard.as_ref() {
                // Poll for messages
                while let Some(msg) = conn.poll_message() {
                    messages.push(msg);
                }
                
                is_connected = conn.is_connected();
            }
        }
        
        // Handle messages outside of the lock
        for msg in messages {
            self.handle_server_message(msg);
        }
        
        // Update connection status
        if is_connected {
            self.connection_status = ConnectionStatus::Connected;
        } else {
            self.connection_status = ConnectionStatus::Disconnected;
        }
    }
    
    fn handle_server_message(&mut self, msg: DebugMessage) {
        match msg {
            DebugMessage::GameState(state) => {
                self.update_game_state(state);
            }
            DebugMessage::AIVisualization { ai_id, data } => {
                self.ai_data.insert(ai_id, data);
            }
            DebugMessage::SimulationPaused(paused) => {
                self.sim_paused = paused;
            }
        }
    }
    
    fn update_game_state(&mut self, msg: ServerMessage) {
        if let ServerMessage::GameState { players, mechs, resources, projectiles } = msg {
            self.game_state.players = players;
            self.game_state.mechs = mechs;
            self.game_state.resources = resources.into_iter()
                .map(|r| (r.id, r))
                .collect();
            self.game_state.projectiles = projectiles;
        }
    }
    
    fn send_command(&self, cmd: DebugCommand) {
        if let Ok(conn_guard) = self.connection.lock() {
            if let Some(conn) = conn_guard.as_ref() {
                conn.send_command(cmd);
            }
        }
    }
}

impl eframe::App for AIDebugApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update from server
        self.update_from_server();
        
        // Top panel with connection status and controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Connection status
                match &self.connection_status {
                    ConnectionStatus::Disconnected => {
                        ui.label("🔴 Disconnected");
                        if ui.button("Connect").clicked() {
                            self.connect_to_server();
                        }
                    }
                    ConnectionStatus::Connecting => {
                        ui.label("🟡 Connecting...");
                    }
                    ConnectionStatus::Connected => {
                        ui.label("🟢 Connected");
                        if ui.button("Disconnect").clicked() {
                            *self.connection.lock().unwrap() = None;
                            self.connection_status = ConnectionStatus::Disconnected;
                        }
                    }
                    ConnectionStatus::Error(e) => {
                        ui.label(format!("🔴 Error: {}", e));
                        if ui.button("Retry").clicked() {
                            self.connect_to_server();
                        }
                    }
                }
                
                ui.separator();
                
                // Simulation controls
                if self.sim_paused {
                    if ui.button("▶ Resume").clicked() {
                        self.send_command(DebugCommand::PauseSimulation(false));
                    }
                } else {
                    if ui.button("⏸ Pause").clicked() {
                        self.send_command(DebugCommand::PauseSimulation(true));
                    }
                }
                
                if ui.button("⏭ Step").clicked() {
                    self.send_command(DebugCommand::StepSimulation);
                }
                
                ui.separator();
                
                ui.label("Speed:");
                if ui.add(egui::Slider::new(&mut self.sim_speed, 0.1..=5.0)).changed() {
                    self.send_command(DebugCommand::SetSimulationSpeed(self.sim_speed));
                }
            });
        });
        
        // Left panel with AI list
        egui::SidePanel::left("ai_list").show(ctx, |ui| {
            ui.heading("AI Players");
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (player_id, player) in &self.game_state.players {
                    if player.name.starts_with("AI_") {
                        let is_selected = self.selected_ai == Some(*player_id);
                        
                        if ui.selectable_label(is_selected, &player.name).clicked() {
                            self.selected_ai = Some(*player_id);
                        }
                        
                        if is_selected {
                            ui.indent("ai_details", |ui| {
                                ui.label(format!("Team: {:?}", player.team));
                                ui.label(format!("Location: {:?}", player.location));
                                
                                if let Some(ai_data) = self.ai_data.get(player_id) {
                                    if let Some(state) = ai_data.ai_states.first() {
                                        ui.label(format!("Hat: {}", state.current_hat));
                                        ui.label(format!("Action: {}", state.current_action));
                                        ui.label(format!("Confidence: {:.1}%", state.confidence * 100.0));
                                    }
                                }
                            });
                        }
                    }
                }
            });
            
            ui.separator();
            
            if ui.button("Add AI").clicked() {
                self.send_command(DebugCommand::AddAI {
                    difficulty: 0.5,
                    personality: "balanced".to_string(),
                });
            }
        });
        
        // Central panel with main view
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ai_id) = self.selected_ai {
                if let Some(ai_data) = self.ai_data.get(&ai_id) {
                    ui.heading(format!("AI Debug: {}", 
                        self.game_state.players.get(&ai_id)
                            .map(|p| p.name.as_str())
                            .unwrap_or("Unknown")));
                    
                    // Tab selection
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.show_communication_graph, true, "Communication");
                        ui.selectable_value(&mut self.show_decision_timeline, true, "Decisions");
                        ui.selectable_value(&mut self.show_performance_metrics, true, "Performance");
                    });
                    
                    ui.separator();
                    
                    // Show selected view
                    if self.show_communication_graph {
                        show_communication_graph(ui, ai_data);
                    }
                    
                    if self.show_decision_timeline {
                        show_decision_timeline(ui, ai_data);
                    }
                    
                    if self.show_performance_metrics {
                        show_performance_metrics(ui, &ai_data.performance_metrics);
                    }
                } else {
                    ui.label("No debug data available for selected AI");
                }
            } else {
                ui.label("Select an AI player from the list to view debug information");
            }
        });
        
        // Request repaint for continuous updates
        ctx.request_repaint();
    }
}

fn show_communication_graph(ui: &mut egui::Ui, ai_data: &AIVisualizationData) {
    ui.heading("Communication Graph");
    
    // Simple text representation for now
    ui.group(|ui| {
        for node in &ai_data.communication_graph.nodes {
            let label = if node.is_captain {
                format!("👑 AI {} (Captain) - {} messages", 
                    &node.ai_id.to_string()[..8], 
                    node.message_count)
            } else {
                format!("🤖 AI {} - {} messages", 
                    &node.ai_id.to_string()[..8], 
                    node.message_count)
            };
            ui.label(label);
        }
        
        ui.separator();
        
        ui.label("Recent Communications:");
        for edge in &ai_data.communication_graph.edges {
            ui.label(format!("{} → {} : {} ({})", 
                &edge.from.to_string()[..8],
                &edge.to.to_string()[..8],
                edge.last_message_type,
                edge.message_count
            ));
        }
    });
}

fn show_decision_timeline(ui: &mut egui::Ui, ai_data: &AIVisualizationData) {
    ui.heading("Decision Timeline");
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        for event in ai_data.decision_timeline.iter().rev() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(&event.timestamp);
                    ui.separator();
                    ui.label(&event.decision_type);
                    ui.separator();
                    ui.label(format!("Confidence: {:.1}%", event.confidence * 100.0));
                });
            });
        }
    });
}

fn show_performance_metrics(ui: &mut egui::Ui, metrics: &AIMetrics) {
    ui.heading("Performance Metrics");
    
    ui.group(|ui| {
        ui.label(format!("Total Decisions: {}", metrics.total_decisions));
        ui.label(format!("Avg Decision Time: {:.2}ms", metrics.average_decision_time_ms));
        ui.label(format!("Decisions/Second: {:.1}", metrics.decisions_per_second));
        ui.label(format!("Messages Sent: {}", metrics.message_count));
        ui.label(format!("Task Success Rate: {:.1}%", metrics.task_success_rate * 100.0));
    });
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum DebugMessage {
    GameState(ServerMessage),
    AIVisualization { ai_id: Uuid, data: AIVisualizationData },
    SimulationPaused(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum DebugCommand {
    PauseSimulation(bool),
    StepSimulation,
    SetSimulationSpeed(f32),
    AddAI { difficulty: f32, personality: String },
    RemoveAI(Uuid),
    RequestAIData(Uuid),
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Mech Battle Arena - AI Debug Client"),
        ..Default::default()
    };
    
    eframe::run_native(
        "AI Debug Client",
        options,
        Box::new(|_cc| Ok(Box::new(AIDebugApp::default()))),
    )
}