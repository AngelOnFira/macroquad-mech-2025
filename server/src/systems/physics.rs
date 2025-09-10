use super::GameSystem;
use crate::game::Game;
use shared::*;
use uuid::Uuid;
use std::collections::VecDeque;

/// Actions that can be queued for physics processing
#[derive(Debug, Clone)]
pub enum PhysicsAction {
    PlayerMovement {
        player_id: Uuid,
        movement: (f32, f32),
        timestamp: f32,
    },
    MechMovement {
        mech_id: Uuid,
        velocity: (f32, f32),
        timestamp: f32,
    },
}

/// Physics system handles object movement, collisions, and physics updates
pub struct PhysicsSystem {
    last_cleanup_time: f32,
    cleanup_interval: f32,
    action_queue: VecDeque<PhysicsAction>,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            last_cleanup_time: 0.0,
            cleanup_interval: 5.0, // Clean up pools every 5 seconds
            action_queue: VecDeque::new(),
        }
    }

    /// Queue a physics action to be processed on the next update
    pub fn queue_action(&mut self, action: PhysicsAction) {
        self.action_queue.push_back(action);
    }

    /// Update mech positions based on their velocity
    fn update_mech_positions(&self, game: &mut Game, delta_time: f32) -> Vec<ServerMessage> {
        let mut messages = Vec::new();

        // Collect mech velocities for testing manager override
        let mut mech_velocities: std::collections::HashMap<uuid::Uuid, (f32, f32)> = game
            .mechs
            .iter()
            .map(|(id, mech)| (*id, mech.velocity))
            .collect();

        // Apply testing manager overrides (for spatial testing)
        game.testing_manager
            .apply_mech_movement_overrides(&mut mech_velocities);

        // Update mech positions with collision checking
        let mut mech_updates = Vec::new();
        
        // First, collect all mechs that want to move
        let mut moving_mechs: Vec<(uuid::Uuid, WorldPos, (f32, f32))> = Vec::new();
        for mech in game.mechs.values() {
            let effective_velocity = mech_velocities
                .get(&mech.id)
                .copied()
                .unwrap_or(mech.velocity);
            
            if effective_velocity.0 != 0.0 || effective_velocity.1 != 0.0 {
                let desired_movement = (
                    effective_velocity.0 * TILE_SIZE * delta_time,
                    effective_velocity.1 * TILE_SIZE * delta_time,
                );
                moving_mechs.push((mech.id, mech.world_position, desired_movement));
            }
        }

        // Create obstacles map first (immutable borrow)
        let mut obstacles_map: std::collections::HashMap<uuid::Uuid, Vec<CollisionShape>> = std::collections::HashMap::new();
        for (mech_id, _, _) in &moving_mechs {
            let mut obstacles = Vec::new();
            for (other_id, other_mech) in game.mechs.iter() {
                if *other_id != *mech_id {
                    obstacles.push(CollisionShape::mech(other_mech.world_position));
                }
            }
            obstacles_map.insert(*mech_id, obstacles);
        }

        // Now apply safe movement (mutable borrow)
        for (mech_id, current_pos, desired_movement) in moving_mechs {
            if let Some(mech) = game.mechs.get_mut(&mech_id) {
                let obstacles = obstacles_map.get(&mech_id).unwrap();
                let mech_shape = CollisionShape::mech(current_pos);
                let safe_movement = CollisionUtils::calculate_safe_movement(
                    current_pos,
                    desired_movement,
                    &mech_shape,
                    obstacles,
                );

                // Apply safe movement
                mech.world_position.x += safe_movement.0;
                mech.world_position.y += safe_movement.1;

                // Keep in bounds
                mech.world_position.x = mech
                    .world_position
                    .x
                    .max(0.0)
                    .min((ARENA_WIDTH_TILES as f32 - MECH_SIZE_TILES as f32) * TILE_SIZE);
                mech.world_position.y = mech
                    .world_position
                    .y
                    .max(0.0)
                    .min((ARENA_HEIGHT_TILES as f32 - MECH_SIZE_TILES as f32) * TILE_SIZE);

                // Update tile position
                let new_tile_pos = mech.world_position.to_tile_pos();
                if new_tile_pos != mech.position {
                    mech.position = new_tile_pos;
                }

                mech_updates.push((mech.id, mech.position, mech.world_position));
            }
        }

        // Send mech position updates
        for (mech_id, position, world_position) in mech_updates {
            messages.push(ServerMessage::MechMoved {
                mech_id,
                position,
                world_position,
            });
        }

        messages
    }

    /// Process queued player movement actions
    fn process_player_movements(&mut self, game: &mut Game, delta_time: f32) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        
        // Process all queued player movement actions
        while let Some(action) = self.action_queue.pop_front() {
            match action {
                PhysicsAction::PlayerMovement { player_id, movement, .. } => {
                    if let Some(updated_location) = self.calculate_player_movement(game, player_id, movement, delta_time) {
                        // Update player position
                        if let Some(player) = game.players.get_mut(&player_id) {
                            player.location = updated_location;
                            
                            // Send movement update
                            messages.push(ServerMessage::PlayerMoved {
                                player_id,
                                location: updated_location,
                            });
                            
                            // Check for tile events at new position (only for OutsideWorld)
                            if let PlayerLocation::OutsideWorld(pos) = updated_location {
                                self.check_tile_events(game, player_id, pos);
                            }
                        }
                    }
                }
                PhysicsAction::MechMovement { mech_id, velocity, .. } => {
                    // Handle mech movement (for future use)
                    if let Some(mech) = game.mechs.get_mut(&mech_id) {
                        mech.velocity = velocity;
                    }
                }
            }
        }
        
        messages
    }

    /// Calculate new player position with collision detection
    fn calculate_player_movement(&self, game: &Game, player_id: Uuid, movement: (f32, f32), delta_time: f32) -> Option<PlayerLocation> {
        let player = game.players.get(&player_id)?;
        let movement_speed = shared::balance::PLAYER_MOVE_SPEED;
        
        // Calculate movement delta
        let delta_x = movement.0 * movement_speed * TILE_SIZE * delta_time;
        let delta_y = movement.1 * movement_speed * TILE_SIZE * delta_time;

        match &player.location {
            PlayerLocation::OutsideWorld(pos) => {
                // Check for collisions and calculate safe movement
                let desired_movement = (delta_x, delta_y);
                let safe_movement = {
                    // Create collision obstacles from all mechs
                    let mut obstacles = Vec::new();
                    for mech in game.mechs.values() {
                        obstacles.push(CollisionShape::mech(mech.world_position));
                    }
                    
                    let player_shape = CollisionShape::player(*pos);
                    CollisionUtils::calculate_safe_movement(
                        *pos,
                        desired_movement,
                        &player_shape,
                        &obstacles,
                    )
                };

                let mut new_pos = *pos;
                new_pos.x += safe_movement.0;
                new_pos.y += safe_movement.1;

                // Keep within world bounds
                new_pos.x = new_pos.x.max(0.0).min((ARENA_WIDTH_TILES as f32) * TILE_SIZE);
                new_pos.y = new_pos.y.max(0.0).min((ARENA_HEIGHT_TILES as f32) * TILE_SIZE);

                Some(PlayerLocation::OutsideWorld(new_pos))
            }
            PlayerLocation::InsideMech { mech_id, pos } => {
                // Convert to local world position, apply movement, then convert back
                let mut new_world_pos = pos.to_local_world();
                new_world_pos.x += delta_x;
                new_world_pos.y += delta_y;

                // Keep within proper mech floor bounds
                let floor_width_pixels = (shared::FLOOR_WIDTH_TILES as f32) * TILE_SIZE;
                let floor_height_pixels = (shared::FLOOR_HEIGHT_TILES as f32) * TILE_SIZE;
                new_world_pos.x = new_world_pos.x.max(0.0).min(floor_width_pixels);
                new_world_pos.y = new_world_pos.y.max(0.0).min(floor_height_pixels);

                // Convert back to MechInteriorPos, preserving floor
                let new_pos = MechInteriorPos::new(pos.floor(), new_world_pos.to_tile());

                Some(PlayerLocation::InsideMech {
                    mech_id: *mech_id,
                    pos: new_pos,
                })
            }
        }
    }

    /// Check for tile events at player position
    fn check_tile_events(&self, game: &mut Game, player_id: Uuid, pos: WorldPos) {
        let tile_pos = pos.to_tile();
        if let Some(tile_content) = game.tile_map.get_world_tile(tile_pos) {
            if let shared::tile_entity::TileContent::Static(static_tile) = tile_content {
                if let Some(tile_event) = static_tile.on_enter(player_id) {
                    // Queue tile event for processing
                    match tile_event {
                        shared::tile_entity::TileEvent::BeginTransition {
                            actor,
                            zone_id: _,
                            transition_type,
                        } => {
                            match transition_type {
                                shared::tile_entity::TransitionType::MechEntrance { stage: _ } => {
                                    // Process mech entry immediately (for now, until we move this to a system)
                                    self.handle_mech_entry(game, actor, tile_pos);
                                }
                                _ => {
                                    // For other tile events, add to tile behavior system queue
                                    if let Some(tile_system) = game
                                        .system_manager
                                        .get_system_mut::<crate::systems::tile_behavior::TileBehaviorSystem>()
                                    {
                                        tile_system.event_queue.push(tile_event);
                                    }
                                }
                            }
                        }
                        _ => {
                            // For other tile events, add to tile behavior system queue
                            if let Some(tile_system) = game
                                .system_manager
                                .get_system_mut::<crate::systems::tile_behavior::TileBehaviorSystem>()
                            {
                                tile_system.event_queue.push(tile_event);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Handle mech entry (temporary - should be moved to a dedicated system later)
    fn handle_mech_entry(&self, game: &mut Game, player_id: Uuid, tile_pos: TilePos) {
        let mech_entry_info = if let Some(player) = game.players.get(&player_id) {
            if let PlayerLocation::OutsideWorld(_) = player.location {
                // Find the mech that owns this door tile
                let mut entry_info = None;
                for (mech_id, mech) in &game.mechs {
                    let doors = shared::coordinates::MechDoorPositions::from_mech_position(mech.position);
                    if tile_pos == doors.left_door || tile_pos == doors.right_door {
                        // Check team access
                        if mech.team == player.team {
                            let entry_world_pos = doors.get_entry_position(tile_pos);
                            let entry_pos = MechInteriorPos::new(0, entry_world_pos.to_tile());
                            entry_info = Some((*mech_id, entry_pos));
                        } else {
                            log::debug!("Player {player_id} denied entry to enemy mech {mech_id}");
                        }
                        break;
                    }
                }
                entry_info
            } else {
                None
            }
        } else {
            None
        };

        // Update player location if entry is allowed
        if let Some((mech_id, entry_pos)) = mech_entry_info {
            if let Some(player_mut) = game.players.get_mut(&player_id) {
                player_mut.location = PlayerLocation::InsideMech {
                    mech_id,
                    pos: entry_pos,
                };
                log::info!("Player {player_id} entered mech {mech_id}");
            }
        }
    }

    /// Update spatial collision manager with current entity positions
    fn update_spatial_collisions(&self, game: &mut Game) {
        // Clear and rebuild spatial collision data
        game.spatial_collision.clear();

        // Add mechs to spatial collision manager
        for mech in game.mechs.values() {
            game.spatial_collision
                .add_mech(mech.id, mech.world_position);
        }

        // Add players to spatial collision manager
        for player in game.players.values() {
            if let PlayerLocation::OutsideWorld(pos) = player.location {
                game.spatial_collision.add_player(player.id, pos);
            }
        }

        // Add resources to spatial collision manager
        for resource in &game.get_resources() {
            let world_pos = resource.position.to_world_pos();
            game.spatial_collision.add_resource(resource.id, world_pos);
        }

        // Add active projectiles to spatial collision manager
        for projectile in game.projectiles.values() {
            if projectile.is_active() {
                game.spatial_collision
                    .add_projectile(projectile.id, projectile.position);
            }
        }
    }

    /// Apply physics constraints and limits
    fn apply_physics_constraints(&self, game: &mut Game) {
        // Apply velocity decay to mechs (friction)
        for mech in game.mechs.values_mut() {
            let decay = 0.95; // 5% velocity decay per frame
            mech.velocity.0 *= decay;
            mech.velocity.1 *= decay;

            // Stop very slow movement to prevent jitter
            if mech.velocity.0.abs() < 0.01 {
                mech.velocity.0 = 0.0;
            }
            if mech.velocity.1.abs() < 0.01 {
                mech.velocity.1 = 0.0;
            }
        }
    }

    /// Clean up pools periodically
    fn cleanup_pools(&mut self, game: &mut Game, current_time: f32) {
        if current_time - self.last_cleanup_time >= self.cleanup_interval {
            game.cleanup_pools();
            self.last_cleanup_time = current_time;
        }
    }
}

impl GameSystem for PhysicsSystem {
    fn update(&mut self, game: &mut Game, delta_time: f32) -> Vec<ServerMessage> {
        let mut messages = Vec::new();

        // Process queued player movements first
        let player_messages = self.process_player_movements(game, delta_time);
        messages.extend(player_messages);

        // Update pooled objects (projectiles and effects)
        let pooled_messages = game.update_pooled_objects(delta_time);
        messages.extend(pooled_messages);

        // Update mech positions
        let mech_messages = self.update_mech_positions(game, delta_time);
        messages.extend(mech_messages);

        // Update spatial collision manager
        self.update_spatial_collisions(game);

        // Apply physics constraints
        self.apply_physics_constraints(game);

        // Log spatial testing information periodically (every 5 seconds in testing mode)
        let current_time = game.tick_count as f32 * delta_time;
        if game.testing_manager.is_testing_mode()
            && game.tick_count % (5.0 / delta_time) as u64 == 0
        {
            game.testing_manager
                .log_spatial_test_info(&game.mechs, &game.players);
        }

        // Clean up pools periodically
        self.cleanup_pools(game, current_time);

        messages
    }

    fn name(&self) -> &'static str {
        "physics"
    }

    fn should_update(&self, _game: &Game) -> bool {
        true // Physics always runs
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for PhysicsSystem {
    fn default() -> Self {
        Self::new()
    }
}
