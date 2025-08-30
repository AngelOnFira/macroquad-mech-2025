use super::GameSystem;
use crate::game::{Game, Resource};
use shared::*;
use uuid::Uuid;

/// Resource system handles resource spawning, collection, and management
pub struct ResourceSystem {
    last_spawn_check: u64,
    spawn_check_interval: u64,
    max_resources: usize,
    resource_spawn_positions: Vec<TilePos>,
}

impl ResourceSystem {
    pub fn new() -> Self {
        Self {
            last_spawn_check: 0,
            spawn_check_interval: 600, // Check every 10 seconds (at 60 FPS)
            max_resources: 20,
            resource_spawn_positions: Self::generate_spawn_positions(),
        }
    }
    
    /// Generate resource spawn positions across the map
    fn generate_spawn_positions() -> Vec<TilePos> {
        let mut positions = Vec::new();
        
        // Add initial spawn positions
        for spawn in INITIAL_RESOURCE_SPAWNS.iter() {
            positions.push(TilePos::new(spawn.0, spawn.1));
        }
        
        // Add additional predefined spawn positions
        let additional_positions = [
            (15, 15), (25, 15), (35, 15), (45, 15),
            (15, 25), (25, 25), (35, 25), (45, 25),
            (15, 35), (25, 35), (35, 35), (45, 35),
            (15, 45), (25, 45), (35, 45),
        ];
        
        for (x, y) in additional_positions {
            positions.push(TilePos::new(x, y));
        }
        
        positions
    }
    
    /// Check if resources need to be spawned
    fn check_resource_spawning(&mut self, game: &mut Game) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        
        // Only check periodically
        if game.tick_count - self.last_spawn_check < self.spawn_check_interval {
            return messages;
        }
        
        self.last_spawn_check = game.tick_count;
        
        // Check if we need more resources
        if game.get_resources().len() < self.max_resources {
            let resources_to_spawn = self.max_resources - game.get_resources().len();
            
            for _ in 0..resources_to_spawn.min(3) { // Spawn max 3 at once
                if let Some(spawn_pos) = self.get_random_spawn_position(game) {
                    let resource_type = self.get_random_resource_type();
                    
                    let resource_id = game.spawn_resource_with_behavior(spawn_pos, resource_type);
                    
                    messages.push(ServerMessage::ResourceSpawned {
                        resource_id,
                        position: spawn_pos,
                        resource_type,
                    });
                    
                    log::info!("Spawned {:?} resource at {:?}", resource_type, spawn_pos);
                }
            }
        }
        
        messages
    }
    
    /// Get a random spawn position that's not occupied
    fn get_random_spawn_position(&self, game: &Game) -> Option<TilePos> {
        let mut available_positions = Vec::new();
        
        for pos in &self.resource_spawn_positions {
            // Check if position is free
            let is_occupied = game.get_resources().iter().any(|r| r.position == *pos);
            if !is_occupied {
                available_positions.push(*pos);
            }
        }
        
        if available_positions.is_empty() {
            return None;
        }
        
        let index = (game.tick_count as usize) % available_positions.len();
        Some(available_positions[index])
    }
    
    /// Get a resource type with round-robin distribution
    fn get_random_resource_type(&self) -> ResourceType {
        // Use a simple cycling approach based on the system's state
        let cycle = (self.last_spawn_check / 100) % 4;
        
        match cycle {
            0 => ResourceType::ScrapMetal,
            1 => ResourceType::Wiring,
            2 => ResourceType::ComputerComponents,
            _ => ResourceType::Batteries,
        }
    }
    
    /// Handle resource pickup logic
    fn handle_resource_pickups(&self, game: &mut Game) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        let mut pickups = Vec::new();
        
        // Use spatial collision manager for efficient pickup detection
        for player in game.players.values() {
            if player.carrying_resource.is_some() {
                continue; // Already carrying something
            }
            
            if let PlayerLocation::OutsideWorld(player_pos) = player.location {
                let nearby_resources = game.spatial_collision.check_player_resource_collisions(
                    player.id, 
                    player_pos
                );
                
                // Pick up the nearest resource
                if let Some(resource_id) = nearby_resources.first() {
                    if let Some(resource) = game.get_resource(*resource_id) {
                        pickups.push((player.id, *resource_id, resource.resource_type));
                    }
                }
            }
        }
        
        // Process pickups
        for (player_id, resource_id, resource_type) in pickups {
            if let Some(player) = game.players.get_mut(&player_id) {
                player.carrying_resource = Some(resource_type);
                game.remove_resource(resource_id);
                
                messages.push(ServerMessage::PlayerPickedUpResource {
                    player_id,
                    resource_type,
                    resource_id,
                });
                
                log::info!("Player {} picked up {:?} resource", player_id, resource_type);
            }
        }
        
        messages
    }
    
    /// Handle resource delivery to mechs
    fn handle_resource_delivery(&self, game: &mut Game) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        let mut deliveries = Vec::new();
        
        for player in game.players.values() {
            if let Some(resource_type) = player.carrying_resource {
                if let PlayerLocation::InsideMech { mech_id, .. } = player.location {
                    // Player is inside a mech with a resource - deliver it
                    deliveries.push((player.id, mech_id, resource_type));
                }
            }
        }
        
        // Process deliveries
        for (player_id, mech_id, resource_type) in deliveries {
            if let Some(player) = game.players.get_mut(&player_id) {
                player.carrying_resource = None;
                
                if let Some(mech) = game.mechs.get_mut(&mech_id) {
                    let current_count = mech.resource_inventory.get(&resource_type).unwrap_or(&0);
                    mech.resource_inventory.insert(resource_type, current_count + 1);
                    
                    messages.push(ServerMessage::ResourceCollected {
                        resource_id: Uuid::new_v4(), // Placeholder
                        player_id,
                    });
                    
                    log::info!("Player {} delivered {:?} to mech {}", player_id, resource_type, mech_id);
                }
            }
        }
        
        messages
    }
    
    /// Balance resource distribution across the map
    fn balance_resource_distribution(&self, game: &Game) -> Vec<TilePos> {
        let mut underrepresented_areas = Vec::new();
        
        // Divide map into quadrants and check resource density
        let quadrants = [
            (0, 0, ARENA_WIDTH_TILES / 2, ARENA_HEIGHT_TILES / 2),
            (ARENA_WIDTH_TILES / 2, 0, ARENA_WIDTH_TILES, ARENA_HEIGHT_TILES / 2),
            (0, ARENA_HEIGHT_TILES / 2, ARENA_WIDTH_TILES / 2, ARENA_HEIGHT_TILES),
            (ARENA_WIDTH_TILES / 2, ARENA_HEIGHT_TILES / 2, ARENA_WIDTH_TILES, ARENA_HEIGHT_TILES),
        ];
        
        for (min_x, min_y, max_x, max_y) in quadrants {
            let resources_in_quadrant = game.get_resources().iter()
                .filter(|r| r.position.x >= min_x && r.position.x < max_x && 
                           r.position.y >= min_y && r.position.y < max_y)
                .count();
            
            if resources_in_quadrant < 2 {
                // This quadrant needs more resources
                let center_x = (min_x + max_x) / 2;
                let center_y = (min_y + max_y) / 2;
                underrepresented_areas.push(TilePos::new(center_x, center_y));
            }
        }
        
        underrepresented_areas
    }
}

impl GameSystem for ResourceSystem {
    fn update(&mut self, game: &mut Game, delta_time: f32) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        
        // Check if new resources need to be spawned
        let spawn_messages = self.check_resource_spawning(game);
        messages.extend(spawn_messages);
        
        // Handle resource pickups
        let pickup_messages = self.handle_resource_pickups(game);
        messages.extend(pickup_messages);
        
        // Handle resource delivery to mechs
        let delivery_messages = self.handle_resource_delivery(game);
        messages.extend(delivery_messages);
        
        messages
    }
    
    fn name(&self) -> &'static str {
        "resource"
    }
    
    fn should_update(&self, _game: &Game) -> bool {
        true // Resource system always runs
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for ResourceSystem {
    fn default() -> Self {
        Self::new()
    }
}