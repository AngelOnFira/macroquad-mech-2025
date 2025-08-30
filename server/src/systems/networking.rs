use super::GameSystem;
use crate::game::Game;
use shared::*;
use uuid::Uuid;

/// Networking system handles periodic game state broadcasts and network optimization
pub struct NetworkingSystem {
    last_full_state_broadcast: u64,
    full_state_interval: u64,
    last_metrics_log: u64,
    metrics_log_interval: u64,
    frame_count: u64,
}

impl NetworkingSystem {
    pub fn new() -> Self {
        Self {
            last_full_state_broadcast: 0,
            full_state_interval: 300, // Broadcast full state every 5 seconds (at 60 FPS)
            last_metrics_log: 0,
            metrics_log_interval: 3600, // Log metrics every minute (at 60 FPS)
            frame_count: 0,
        }
    }
    
    /// Determine if a full game state broadcast is needed
    fn should_broadcast_full_state(&self, game: &Game) -> bool {
        game.tick_count - self.last_full_state_broadcast >= self.full_state_interval
    }
    
    /// Create optimized state updates instead of full game state
    fn create_incremental_updates(&self, game: &Game) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        
        // Only send updates for entities that have changed
        // This is a simplified version - in a real implementation,
        // you'd track dirty flags for each entity
        
        // For now, we'll send periodic position updates for moving mechs
        if self.frame_count % 10 == 0 { // Every 10 frames
            for mech in game.mechs.values() {
                if mech.velocity.0 != 0.0 || mech.velocity.1 != 0.0 {
                    messages.push(ServerMessage::MechMoved {
                        mech_id: mech.id,
                        position: mech.position,
                        world_position: mech.world_position,
                    });
                }
            }
        }
        
        messages
    }
    
    /// Log performance metrics
    fn log_performance_metrics(&mut self, game: &Game) {
        if game.tick_count - self.last_metrics_log >= self.metrics_log_interval {
            self.last_metrics_log = game.tick_count;
            
            let pool_stats = game.get_pool_stats();
            
            log::info!("=== Game Performance Metrics ===");
            log::info!("Tick: {}", game.tick_count);
            log::info!("Players: {}", game.players.len());
            log::info!("Mechs: {}", game.mechs.len());
            log::info!("Resources: {}", game.get_resources().len());
            log::info!("Active Projectiles: {}", game.projectiles.len());
            log::info!("Active Effects: {}", game.active_effects.len());
            log::info!("Pool Stats - Projectiles: {}/{}, Effects: {}/{}",
                pool_stats.projectiles_available,
                pool_stats.projectiles_max,
                pool_stats.effects_available,
                pool_stats.effects_max
            );
            log::info!("==============================");
        }
    }
    
    /// Handle network bandwidth optimization
    fn optimize_bandwidth(&self, messages: &mut Vec<ServerMessage>) {
        // Remove duplicate messages of the same type for the same entity
        // This is a simplified version of message deduplication
        
        let mut seen_mech_moves = std::collections::HashSet::new();
        let mut seen_player_moves = std::collections::HashSet::new();
        
        messages.retain(|msg| {
            match msg {
                ServerMessage::MechMoved { mech_id, .. } => {
                    if seen_mech_moves.contains(mech_id) {
                        false // Remove duplicate
                    } else {
                        seen_mech_moves.insert(*mech_id);
                        true
                    }
                }
                ServerMessage::PlayerMoved { player_id, .. } => {
                    if seen_player_moves.contains(player_id) {
                        false // Remove duplicate
                    } else {
                        seen_player_moves.insert(*player_id);
                        true
                    }
                }
                _ => true, // Keep all other messages
            }
        });
    }
    
    /// Compress game state for large broadcasts
    fn compress_game_state(&self, _game: &Game) -> Option<ServerMessage> {
        // TODO: Implement game state compression
        // This could involve:
        // - Delta compression (only send changes since last state)
        // - Spatial culling (only send entities in view)
        // - Level of detail (reduce precision for distant entities)
        None
    }
    
    /// Handle connection quality adaptation
    fn adapt_to_connection_quality(&self, _game: &Game) -> NetworkAdaptation {
        // TODO: Implement adaptive networking based on connection quality
        // This could involve:
        // - Reducing update frequency for poor connections
        // - Switching to lower fidelity updates
        // - Implementing client-side prediction compensation
        
        NetworkAdaptation {
            update_frequency: 1.0, // Normal frequency
            precision_level: PrecisionLevel::High,
            compression_enabled: false,
        }
    }
    
    /// Validate message integrity
    fn validate_messages(&self, messages: &[ServerMessage]) -> bool {
        // Basic message validation
        for message in messages {
            match message {
                ServerMessage::MechMoved { mech_id, position, world_position } => {
                    // Validate that positions are reasonable
                    if position.x < 0 || position.y < 0 || 
                       position.x >= ARENA_WIDTH_TILES || position.y >= ARENA_HEIGHT_TILES {
                        log::warn!("Invalid mech position in message: {:?}", position);
                        return false;
                    }
                    
                    if world_position.x < 0.0 || world_position.y < 0.0 ||
                       world_position.x >= (ARENA_WIDTH_TILES as f32 * TILE_SIZE) ||
                       world_position.y >= (ARENA_HEIGHT_TILES as f32 * TILE_SIZE) {
                        log::warn!("Invalid mech world position in message: {:?}", world_position);
                        return false;
                    }
                }
                _ => {} // Other message types pass validation for now
            }
        }
        
        true
    }
}

impl GameSystem for NetworkingSystem {
    fn update(&mut self, game: &mut Game, delta_time: f32) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        self.frame_count += 1;
        
        // Check if we need to broadcast full game state
        if self.should_broadcast_full_state(game) {
            self.last_full_state_broadcast = game.tick_count;
            messages.push(game.get_full_state());
        } else {
            // Send incremental updates
            let incremental = self.create_incremental_updates(game);
            messages.extend(incremental);
        }
        
        // Optimize bandwidth usage
        self.optimize_bandwidth(&mut messages);
        
        // Validate messages before sending
        if !self.validate_messages(&messages) {
            log::error!("Message validation failed, dropping invalid messages");
            messages.clear();
        }
        
        // Log performance metrics periodically
        self.log_performance_metrics(game);
        
        messages
    }
    
    fn name(&self) -> &'static str {
        "networking"
    }
    
    fn should_update(&self, game: &Game) -> bool {
        // Networking runs every few frames instead of every frame
        game.tick_count % 3 == 0
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for NetworkingSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Network adaptation configuration
#[derive(Debug, Clone)]
pub struct NetworkAdaptation {
    pub update_frequency: f32,
    pub precision_level: PrecisionLevel,
    pub compression_enabled: bool,
}

/// Precision levels for network updates
#[derive(Debug, Clone, Copy)]
pub enum PrecisionLevel {
    Low,    // Reduced precision for poor connections
    Medium, // Standard precision
    High,   // Full precision for good connections
}