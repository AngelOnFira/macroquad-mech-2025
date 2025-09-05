use super::GameSystem;
use crate::game::Game;
use shared::*;
use uuid::Uuid;

/// Combat system handles weapon firing, projectile collisions, and damage
pub struct CombatSystem {
    last_damage_tick: u64,
}

impl CombatSystem {
    pub fn new() -> Self {
        Self {
            last_damage_tick: 0,
        }
    }

    /// Check projectile collisions with mechs
    fn check_projectile_collisions(&self, game: &mut Game) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        let mut hits = Vec::new();

        for projectile in game.projectiles.values() {
            if !projectile.is_active() {
                continue;
            }

            let proj_tile = projectile.position.to_tile_pos();

            for mech in game.mechs.values() {
                // Don't hit the mech that fired the projectile
                if mech.id == projectile.owner_mech_id {
                    continue;
                }

                let mech_min = mech.position;
                let mech_max = mech.position.offset(MECH_SIZE_TILES, MECH_SIZE_TILES);

                // Check if projectile is within mech bounds
                if proj_tile.x >= mech_min.x
                    && proj_tile.x <= mech_max.x
                    && proj_tile.y >= mech_min.y
                    && proj_tile.y <= mech_max.y
                {
                    hits.push((projectile.id, mech.id, projectile.damage));
                    break;
                }
            }
        }

        // Process hits
        for (proj_id, mech_id, damage) in hits {
            // Remove projectile and return to pool
            if let Some(mut projectile) = game.projectiles.remove(&proj_id) {
                projectile.reset();
                game.pool_manager.return_projectile(projectile);
            }

            // Apply damage to mech
            let (explosion_pos, health_remaining, is_destroyed) = {
                if let Some(mech) = game.mechs.get_mut(&mech_id) {
                    // Apply damage to shield first, then health
                    let shield_damage = damage.min(mech.shield);
                    mech.shield -= shield_damage;
                    let health_damage = damage - shield_damage;
                    mech.health = mech.health.saturating_sub(health_damage);

                    (mech.world_position, mech.health, mech.health == 0)
                } else {
                    continue;
                }
            };

            // Create explosion effect after releasing the mutable borrow
            let explosion_color = (1.0, 0.5, 0.0, 1.0); // Orange explosion
            game.create_effect(
                EffectType::Explosion,
                explosion_pos,
                1.0, // 1 second duration
                1.0, // Full intensity
                explosion_color,
            );

            messages.push(ServerMessage::MechDamaged {
                mech_id,
                damage,
                health_remaining,
            });

            messages.push(ServerMessage::ProjectileHit {
                projectile_id: proj_id,
                hit_mech_id: Some(mech_id),
                damage_dealt: damage,
            });

            // Check if mech is destroyed
            if is_destroyed {
                self.handle_mech_destroyed(game, mech_id, &mut messages);
            }
        }

        messages
    }

    /// Handle mech destruction
    fn handle_mech_destroyed(
        &self,
        game: &mut Game,
        mech_id: Uuid,
        messages: &mut Vec<ServerMessage>,
    ) {
        // Eject all players from destroyed mech
        let mut players_to_eject = Vec::new();

        for (player_id, player) in game.players.iter() {
            if let PlayerLocation::InsideMech {
                mech_id: player_mech_id,
                ..
            } = player.location
            {
                if player_mech_id == mech_id {
                    players_to_eject.push(*player_id);
                }
            }
        }

        // Eject players to their team spawn
        for player_id in players_to_eject {
            if let Some(player) = game.players.get_mut(&player_id) {
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

                player.location = PlayerLocation::OutsideWorld(spawn_pos);
                player.carrying_resource = None;
                player.operating_station = None;

                messages.push(ServerMessage::PlayerMoved {
                    player_id,
                    location: player.location,
                });
            }
        }

        // Create large explosion effect
        if let Some(mech) = game.mechs.get(&mech_id) {
            let explosion_pos = mech.world_position;
            let explosion_color = (1.0, 0.2, 0.0, 1.0); // Red explosion
            game.create_effect(
                EffectType::Explosion,
                explosion_pos,
                3.0, // 3 second duration
                2.0, // High intensity
                explosion_color,
            );
        }

        // TODO: Respawn mech after some time
        // For now, just log the destruction
        log::info!("Mech {mech_id} destroyed");
    }

    /// Apply damage over time effects
    fn apply_damage_over_time(&mut self, game: &mut Game) -> Vec<ServerMessage> {
        let mut messages = Vec::new();

        // Skip if not enough ticks have passed
        if game.tick_count - self.last_damage_tick < 60 {
            return messages;
        }

        self.last_damage_tick = game.tick_count;

        // Apply shield regeneration
        for mech in game.mechs.values_mut() {
            if mech.shield < mech.max_shield {
                let regen_amount = (mech.max_shield / 20).max(1); // 5% per second
                mech.shield = (mech.shield + regen_amount).min(mech.max_shield);

                messages.push(ServerMessage::MechShieldChanged {
                    mech_id: mech.id,
                    shield: mech.shield,
                });
            }
        }

        messages
    }

    /// Process weapon range and targeting
    fn process_weapon_targeting(
        &self,
        game: &Game,
        firing_mech_id: Uuid,
        weapon_range: f32,
    ) -> Option<Uuid> {
        let firing_mech = game.mechs.get(&firing_mech_id)?;

        // Find nearest enemy mech within range
        let mut nearest_enemy = None;
        let mut nearest_distance = f32::MAX;

        for mech in game.mechs.values() {
            if mech.id == firing_mech_id || mech.team == firing_mech.team {
                continue;
            }

            let distance = firing_mech.world_position.distance_to(mech.world_position);
            if distance <= weapon_range && distance < nearest_distance {
                nearest_distance = distance;
                nearest_enemy = Some(mech.id);
            }
        }

        nearest_enemy
    }

    /// Create combat effects (laser beams, muzzle flashes, etc.)
    fn create_combat_effects(
        &self,
        game: &mut Game,
        weapon_type: StationType,
        from_pos: WorldPos,
        to_pos: Option<WorldPos>,
    ) {
        match weapon_type {
            StationType::WeaponLaser => {
                if let Some(target_pos) = to_pos {
                    // Create laser beam effect
                    game.create_effect(
                        EffectType::LaserBeam,
                        from_pos,
                        0.2,                  // 0.2 second duration
                        1.0,                  // Full intensity
                        (0.0, 1.0, 0.0, 1.0), // Green laser
                    );

                    // Create hit effect at target
                    game.create_effect(
                        EffectType::Damage,
                        target_pos,
                        0.5,                  // 0.5 second duration
                        0.8,                  // High intensity
                        (1.0, 0.0, 0.0, 1.0), // Red hit effect
                    );
                }
            }
            StationType::WeaponProjectile => {
                // Create muzzle flash effect
                game.create_effect(
                    EffectType::Explosion,
                    from_pos,
                    0.1,                  // 0.1 second duration
                    0.5,                  // Medium intensity
                    (1.0, 1.0, 0.0, 1.0), // Yellow muzzle flash
                );
            }
            _ => {}
        }
    }
}

impl GameSystem for CombatSystem {
    fn update(&mut self, game: &mut Game, delta_time: f32) -> Vec<ServerMessage> {
        let mut messages = Vec::new();

        // Check projectile collisions
        let collision_messages = self.check_projectile_collisions(game);
        messages.extend(collision_messages);

        // Apply damage over time effects
        let dot_messages = self.apply_damage_over_time(game);
        messages.extend(dot_messages);

        messages
    }

    fn name(&self) -> &'static str {
        "combat"
    }

    fn should_update(&self, _game: &Game) -> bool {
        true // Combat always runs
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for CombatSystem {
    fn default() -> Self {
        Self::new()
    }
}
