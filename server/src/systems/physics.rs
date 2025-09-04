use super::GameSystem;
use crate::game::Game;
use shared::*;

/// Physics system handles object movement, collisions, and physics updates
pub struct PhysicsSystem {
    last_cleanup_time: f32,
    cleanup_interval: f32,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            last_cleanup_time: 0.0,
            cleanup_interval: 5.0, // Clean up pools every 5 seconds
        }
    }

    /// Update mech positions based on their velocity
    fn update_mech_positions(&self, game: &mut Game, delta_time: f32) -> Vec<ServerMessage> {
        let mut messages = Vec::new();

        // Update mech positions
        let mut mech_updates = Vec::new();
        for mech in game.mechs.values_mut() {
            if mech.velocity.0 != 0.0 || mech.velocity.1 != 0.0 {
                // Update world position
                mech.world_position.x += mech.velocity.0 * TILE_SIZE * delta_time;
                mech.world_position.y += mech.velocity.1 * TILE_SIZE * delta_time;

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

    /// Check for collisions between players and mechs
    fn check_mech_player_collisions(&self, game: &mut Game) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        let mut killed_players = Vec::new();

        for (player_id, player) in game.players.iter() {
            if let PlayerLocation::OutsideWorld(player_pos) = player.location {
                for mech in game.mechs.values() {
                    // Check if player is within mech bounds
                    let mech_min_x = mech.world_position.x;
                    let mech_max_x = mech.world_position.x + (MECH_SIZE_TILES as f32 * TILE_SIZE);
                    let mech_min_y = mech.world_position.y;
                    let mech_max_y = mech.world_position.y + (MECH_SIZE_TILES as f32 * TILE_SIZE);

                    if player_pos.x >= mech_min_x
                        && player_pos.x <= mech_max_x
                        && player_pos.y >= mech_min_y
                        && player_pos.y <= mech_max_y
                    {
                        // Player was run over!
                        killed_players.push(*player_id);
                        break;
                    }
                }
            }
        }

        // Handle killed players
        for player_id in killed_players {
            if let Some(player) = game.players.get(&player_id) {
                // Respawn at team spawn
                let spawn_pos = match player.team {
                    TeamId::Red => WorldPos::new(
                        RED_PLAYER_SPAWN.0 * TILE_SIZE,
                        RED_PLAYER_SPAWN.1 * TILE_SIZE,
                    ),
                    TeamId::Blue => WorldPos::new(
                        BLUE_PLAYER_SPAWN.0 * TILE_SIZE,
                        BLUE_PLAYER_SPAWN.1 * TILE_SIZE,
                    ),
                };

                messages.push(ServerMessage::PlayerKilled {
                    player_id,
                    killer: None, // Killed by mech
                    respawn_position: spawn_pos,
                });

                // Reset player state
                if let Some(player) = game.players.get_mut(&player_id) {
                    player.location = PlayerLocation::OutsideWorld(spawn_pos);
                    player.carrying_resource = None;
                    player.operating_station = None;
                }
            }
        }

        messages
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

        // Update pooled objects (projectiles and effects)
        let pooled_messages = game.update_pooled_objects(delta_time);
        messages.extend(pooled_messages);

        // Update mech positions
        let mech_messages = self.update_mech_positions(game, delta_time);
        messages.extend(mech_messages);

        // Check for mech-player collisions
        let collision_messages = self.check_mech_player_collisions(game);
        messages.extend(collision_messages);

        // Update spatial collision manager
        self.update_spatial_collisions(game);

        // Apply physics constraints
        self.apply_physics_constraints(game);

        // Clean up pools periodically
        let current_time = game.tick_count as f32 * delta_time;
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
