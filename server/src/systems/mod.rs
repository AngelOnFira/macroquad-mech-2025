pub mod physics;
pub mod combat;
pub mod resource;
pub mod networking;
pub mod ai;
pub mod tile_behavior;

use crate::game::Game;
use shared::ServerMessage;
use uuid::Uuid;
use tokio::sync::broadcast;

/// Trait for game systems that can be updated each frame
pub trait GameSystem {
    /// Update the system with the given delta time
    /// Returns a list of server messages to send to clients
    fn update(&mut self, game: &mut Game, delta_time: f32) -> Vec<ServerMessage>;
    
    /// Get the name of this system for debugging
    fn name(&self) -> &'static str;
    
    /// Check if this system should be updated this frame
    fn should_update(&self, _game: &Game) -> bool {
        true
    }
    
    /// Get self as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Manager for all game systems
pub struct SystemManager {
    systems: Vec<Box<dyn GameSystem + Send + Sync>>,
    tick_count: u64,
}

impl SystemManager {
    /// Create a new system manager with default systems
    pub fn new() -> Self {
        let mut manager = Self {
            systems: Vec::new(),
            tick_count: 0,
        };
        
        // Register default systems in order of execution
        manager.register_system(Box::new(tile_behavior::TileBehaviorSystem::new()));
        manager.register_system(Box::new(physics::PhysicsSystem::new()));
        manager.register_system(Box::new(combat::CombatSystem::new()));
        manager.register_system(Box::new(resource::ResourceSystem::new()));
        manager.register_system(Box::new(networking::NetworkingSystem::new()));
        manager.register_system(Box::new(ai::AISystem::new()));
        
        manager
    }
    
    /// Register a new system
    pub fn register_system(&mut self, system: Box<dyn GameSystem + Send + Sync>) {
        self.systems.push(system);
    }
    
    /// Update all systems for one frame
    pub fn update_all(&mut self, game: &mut Game, delta_time: f32) -> Vec<ServerMessage> {
        let mut all_messages = Vec::new();
        
        self.tick_count += 1;
        
        for system in &mut self.systems {
            if system.should_update(game) {
                let messages = system.update(game, delta_time);
                all_messages.extend(messages);
            }
        }
        
        all_messages
    }
    
    /// Update a specific system by name
    pub fn update_system(&mut self, game: &mut Game, system_name: &str, delta_time: f32) -> Vec<ServerMessage> {
        for system in &mut self.systems {
            if system.name() == system_name {
                return system.update(game, delta_time);
            }
        }
        Vec::new()
    }
    
    /// Get system count
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }
    
    /// Get current tick count
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }
    
    /// Get a mutable reference to a specific system by type
    pub fn get_system_mut<T: GameSystem + 'static>(&mut self) -> Option<&mut T> {
        self.systems.iter_mut()
            .find_map(|system| {
                system.as_any_mut().downcast_mut::<T>()
            })
    }
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new()
    }
}

// Ensure SystemManager can be taken with std::mem::take
unsafe impl Send for SystemManager {}
unsafe impl Sync for SystemManager {}

/// System update frequency configuration
#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub physics_hz: f32,
    pub combat_hz: f32,
    pub resource_hz: f32,
    pub networking_hz: f32,
    pub ai_hz: f32,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            physics_hz: 60.0,      // 60 FPS physics
            combat_hz: 30.0,       // 30 FPS combat updates
            resource_hz: 10.0,     // 10 FPS resource spawning
            networking_hz: 20.0,   // 20 FPS network broadcasts
            ai_hz: 20.0,           // 20 FPS AI updates
        }
    }
}

/// System performance metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub system_name: String,
    pub updates_per_second: f32,
    pub average_update_time_ms: f32,
    pub last_update_time_ms: f32,
}

/// Manager for tracking system performance
pub struct MetricsManager {
    metrics: std::collections::HashMap<String, SystemMetrics>,
    last_update_time: std::time::Instant,
}

impl MetricsManager {
    pub fn new() -> Self {
        Self {
            metrics: std::collections::HashMap::new(),
            last_update_time: std::time::Instant::now(),
        }
    }
    
    /// Record an update for a system
    pub fn record_update(&mut self, system_name: &str, update_time_ms: f32) {
        let metrics = self.metrics.entry(system_name.to_string())
            .or_insert_with(|| SystemMetrics {
                system_name: system_name.to_string(),
                updates_per_second: 0.0,
                average_update_time_ms: 0.0,
                last_update_time_ms: 0.0,
            });
        
        metrics.last_update_time_ms = update_time_ms;
        
        // Simple rolling average
        metrics.average_update_time_ms = 
            (metrics.average_update_time_ms * 0.9) + (update_time_ms * 0.1);
    }
    
    /// Get metrics for all systems
    pub fn get_all_metrics(&self) -> Vec<&SystemMetrics> {
        self.metrics.values().collect()
    }
    
    /// Get metrics for a specific system
    pub fn get_metrics(&self, system_name: &str) -> Option<&SystemMetrics> {
        self.metrics.get(system_name)
    }
}

impl Default for MetricsManager {
    fn default() -> Self {
        Self::new()
    }
}