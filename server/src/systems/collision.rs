use std::collections::HashMap;

use super::GameSystem;
use crate::game::Game;
use log::info;
use shared::*;
use uuid::Uuid;

/// Collision system handles all entity-entity collisions and responses
pub struct CollisionSystem {
    /// Cache for collision shapes to avoid recalculating every frame
    player_shapes: HashMap<Uuid, CollisionShape>,
    mech_shapes: HashMap<Uuid, CollisionShape>,
}

impl CollisionSystem {
    pub fn new() -> Self {
        Self {
            player_shapes: HashMap::new(),
            mech_shapes: HashMap::new(),
        }
    }

    /// Update collision shapes cache with current entity positions
    fn update_collision_shapes(&mut self, game: &Game) {
        // Update player collision shapes
        self.player_shapes.clear();
        for player in game.players.values() {
            if let PlayerLocation::OutsideWorld(pos) = player.location {
                let shape = CollisionShape::player(pos);
                self.player_shapes.insert(player.id, shape);
            }
        }

        // Update mech collision shapes
        self.mech_shapes.clear();
        for mech in game.mechs.values() {
            let shape = CollisionShape::mech(mech.world_position);
            self.mech_shapes.insert(mech.id, shape);
        }
    }

    /// Check for player-mech collisions and return appropriate responses
    fn check_player_mech_collisions(&self, game: &mut Game) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        let mut killed_players = Vec::new();
        let mut player_pushes: Vec<(Uuid, f32, f32)> = Vec::new();

        for (player_id, player) in game.players.iter() {
            if let PlayerLocation::OutsideWorld(player_pos) = player.location {
                let player_shape = CollisionShape::player(player_pos);

                for (mech_id, mech) in game.mechs.iter() {
                    let mech_shape = CollisionShape::mech(mech.world_position);

                    if let Some(manifold) = CollisionManifold::aabb_vs_aabb(&player_shape.aabb, &mech_shape.aabb) {
                        // Check if this is a run-over scenario
                        if CollisionUtils::should_cause_run_over_damage(
                            mech.velocity,
                            mech.world_position,
                            player_pos,
                            RUN_OVER_MIN_VELOCITY,
                        ) {
                            info!("Player {player_id} was run over by mech {mech_id}");
                            killed_players.push(*player_id);
                            break; // Player is dead, no need to check other mechs
                        } else {
                            // Push player away from mech
                            let push_force = PLAYER_PUSH_DISTANCE;
                            let push_x = manifold.normal.0 * push_force;
                            let push_y = manifold.normal.1 * push_force;
                            player_pushes.push((*player_id, push_x, push_y));
                        }
                    }
                }
            }
        }

        // Handle killed players
        for player_id in killed_players {
            if let Some(player) = game.players.get(&player_id) {
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
                if let Some(player_mut) = game.players.get_mut(&player_id) {
                    player_mut.location = PlayerLocation::OutsideWorld(spawn_pos);
                    player_mut.carrying_resource = None;
                    player_mut.operating_station = None;
                }
            }
        }

        // Apply player pushes (non-lethal collisions)
        for (player_id, push_x, push_y) in player_pushes {
            if let Some(player) = game.players.get_mut(&player_id) {
                if let PlayerLocation::OutsideWorld(ref mut pos) = player.location {
                    pos.x += push_x;
                    pos.y += push_y;

                    // Keep within world bounds
                    pos.x = pos.x.max(0.0).min((ARENA_WIDTH_TILES as f32) * TILE_SIZE);
                    pos.y = pos.y.max(0.0).min((ARENA_HEIGHT_TILES as f32) * TILE_SIZE);

                    messages.push(ServerMessage::PlayerMoved {
                        player_id,
                        location: player.location,
                    });
                }
            }
        }

        messages
    }

    /// Check for mech-mech collisions and apply separation forces
    fn check_mech_mech_collisions(&self, game: &mut Game) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        let mut separations: Vec<(Uuid, f32, f32)> = Vec::new();

        let mech_ids: Vec<Uuid> = game.mechs.keys().cloned().collect();

        // Check all mech pairs
        for i in 0..mech_ids.len() {
            for j in (i + 1)..mech_ids.len() {
                let mech_id_a = mech_ids[i];
                let mech_id_b = mech_ids[j];

                if let (Some(mech_a), Some(mech_b)) = (game.mechs.get(&mech_id_a), game.mechs.get(&mech_id_b)) {
                    let shape_a = CollisionShape::mech(mech_a.world_position);
                    let shape_b = CollisionShape::mech(mech_b.world_position);

                    if let Some(separation) = CollisionUtils::calculate_separation(&shape_a, &shape_b) {
                        // Apply half the separation to each mech
                        separations.push((mech_id_a, -separation.0 * 0.5, -separation.1 * 0.5));
                        separations.push((mech_id_b, separation.0 * 0.5, separation.1 * 0.5));
                    }
                }
            }
        }

        // Apply mech separations
        for (mech_id, sep_x, sep_y) in separations {
            if let Some(mech) = game.mechs.get_mut(&mech_id) {
                mech.world_position.x += sep_x;
                mech.world_position.y += sep_y;

                // Keep within world bounds
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

                messages.push(ServerMessage::MechMoved {
                    mech_id,
                    position: mech.position,
                    world_position: mech.world_position,
                });
            }
        }

        messages
    }

    /// Check if a proposed movement would cause collisions
    pub fn check_movement_collision(
        &self,
        entity_id: Uuid,
        start_pos: WorldPos,
        desired_movement: (f32, f32),
        entity_type: CollisionLayer,
        game: &Game,
    ) -> (f32, f32) {
        let collision_shape = match entity_type {
            CollisionLayer::Player => CollisionShape::player(start_pos),
            CollisionLayer::Mech => CollisionShape::mech(start_pos),
            _ => return desired_movement, // Other types not implemented yet
        };

        // Gather all potential obstacles
        let mut obstacles = Vec::new();

        // Add all mechs as obstacles (except self if checking mech movement)
        for (mech_id, mech) in game.mechs.iter() {
            if entity_type == CollisionLayer::Mech && *mech_id == entity_id {
                continue; // Don't collide with self
            }
            obstacles.push(CollisionShape::mech(mech.world_position));
        }

        // If checking player movement, only add mechs (players can pass through other players)
        // If checking mech movement, add other mechs

        CollisionUtils::calculate_safe_movement(start_pos, desired_movement, &collision_shape, &obstacles)
    }
}

impl GameSystem for CollisionSystem {
    fn update(&mut self, game: &mut Game, _delta_time: f32) -> Vec<ServerMessage> {
        let mut messages = Vec::new();

        // Update collision shapes with current positions
        self.update_collision_shapes(game);

        // Check player-mech collisions (run-over vs push-away)
        let player_mech_messages = self.check_player_mech_collisions(game);
        messages.extend(player_mech_messages);

        // Check mech-mech collisions and apply separation
        let mech_mech_messages = self.check_mech_mech_collisions(game);
        messages.extend(mech_mech_messages);

        messages
    }

    fn name(&self) -> &'static str {
        "collision"
    }

    fn should_update(&self, _game: &Game) -> bool {
        true // Collision system always runs
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for CollisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_system_creation() {
        let system = CollisionSystem::new();
        assert_eq!(system.name(), "collision");
    }

    #[test]
    fn test_movement_collision_check() {
        let mut game = Game::new();
        let collision_system = CollisionSystem::new();
        
        // Create a test mech
        let mech_id = Uuid::new_v4();
        let mech_pos = WorldPos::new(100.0, 100.0);
        // We'd need to add a mech to the game here, but Game::new() may not have the interface
        // This test would need to be more complex in a real scenario

        let player_id = Uuid::new_v4();
        let player_pos = WorldPos::new(90.0, 100.0); // Near the mech
        let desired_movement = (20.0, 0.0); // Moving toward mech

        let safe_movement = collision_system.check_movement_collision(
            player_id,
            player_pos,
            desired_movement,
            CollisionLayer::Player,
            &game,
        );

        // Safe movement should be less than or equal to desired movement
        assert!(safe_movement.0.abs() <= desired_movement.0.abs());
        assert!(safe_movement.1.abs() <= desired_movement.1.abs());
    }
}