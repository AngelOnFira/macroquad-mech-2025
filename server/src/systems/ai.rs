use crate::{systems::GameSystem, Game};
use ai::{AICommand, AIManager, GameView};
use shared::*;
use std::collections::HashMap;
use uuid::Uuid;

/// AI System that manages AI players
pub struct AISystem {
    ai_manager: AIManager,
    ai_players: HashMap<Uuid, AIPlayerInfo>,
}

struct AIPlayerInfo {
    name: String,
    team: TeamId,
    mech_id: Option<Uuid>,
}

impl AISystem {
    pub fn new() -> Self {
        Self {
            ai_manager: AIManager::new(Default::default()),
            ai_players: HashMap::new(),
        }
    }

    /// Get debug info for a specific AI
    pub fn get_ai_debug_info(&self, ai_id: Uuid) -> Option<ai::AIDebugInfo> {
        self.ai_manager.get_debug_info(ai_id)
    }

    /// Add an AI player to the manager
    pub fn add_ai_player(
        &mut self,
        difficulty: f32,
        personality: Option<ai::Personality>,
        red_count: usize,
        blue_count: usize,
    ) -> (Uuid, crate::game::Player) {
        let personality = personality.unwrap_or(ai::Personality::Balanced);
        let ai_id = self.ai_manager.add_ai(personality, difficulty);

        // Create player name
        let name = format!("AI_{}", personality.name_suffix());

        // Determine team (balance teams)
        let team = if red_count <= blue_count {
            TeamId::Red
        } else {
            TeamId::Blue
        };

        // Create player
        let player = crate::game::Player {
            id: ai_id,
            name: name.clone(),
            location: PlayerLocation::OutsideWorld(WorldPos { x: 0.0, y: 0.0 }),
            team,
            carrying_resource: None,
            operating_station: None,
        };

        // Track AI info
        self.ai_players.insert(
            ai_id,
            AIPlayerInfo {
                name,
                team,
                mech_id: None,
            },
        );

        (ai_id, player)
    }

    /// Remove an AI player
    pub fn remove_ai_player(&mut self, ai_id: Uuid) -> bool {
        self.ai_manager.remove_ai(ai_id);
        self.ai_players.remove(&ai_id).is_some()
    }

    /// Convert game state to AI view
    fn create_game_view(&self, game: &Game, ai_id: Uuid) -> GameView {
        let player = game.players.get(&ai_id).unwrap();

        // Get all players as PlayerView
        let players: Vec<ai::PlayerView> = game
            .players
            .iter()
            .map(|(id, p)| {
                // Get the operating station type if player is operating a station
                let operating_station = if let Some(station_id) = p.operating_station {
                    // Find the station in any mech
                    game.mechs.values().find_map(|mech| {
                        mech.stations
                            .get(&station_id)
                            .map(|station| station.station_type)
                    })
                } else {
                    None
                };

                ai::PlayerView {
                    id: *id,
                    name: p.name.clone(),
                    team: p.team,
                    location: p.location,
                    carrying_resource: p.carrying_resource,
                    operating_station,
                    is_self: *id == ai_id,
                }
            })
            .collect();

        // Get all mechs as MechView
        let mechs: Vec<ai::MechView> = game
            .mechs
            .iter()
            .map(|(id, m)| {
                // Convert stations to StationView
                let stations: Vec<ai::StationView> = m
                    .stations
                    .values()
                    .map(|s| ai::StationView {
                        id: s.id,
                        station_type: s.station_type,
                        operated_by: s.operated_by,
                        position: s.position,
                        floor: s.floor,
                    })
                    .collect();

                ai::MechView {
                    id: *id,
                    team: m.team,
                    position: m.world_position,
                    health: m.health,
                    shield: m.shield,
                    velocity: m.velocity,
                    stations,
                    resource_inventory: m.resource_inventory.clone(),
                }
            })
            .collect();

        // Get visible resources
        let resources: Vec<ai::ResourceView> = game
            .get_resources()
            .iter()
            .map(|r| ai::ResourceView {
                id: r.id,
                position: r.position.to_world_pos(),
                resource_type: r.resource_type,
            })
            .collect();

        // Get visible projectiles
        let projectiles: Vec<ai::ProjectileView> = game
            .projectiles
            .iter()
            .map(|(id, p)| {
                // Find the team of the owner mech
                let owner_team = game
                    .mechs
                    .get(&p.owner_mech_id)
                    .map(|m| m.team)
                    .unwrap_or(TeamId::Red); // Default if mech not found

                ai::ProjectileView {
                    id: *id,
                    position: p.position,
                    velocity: p.velocity,
                    owner_team,
                }
            })
            .collect();

        // Get team info
        let team_members: Vec<Uuid> = game
            .players
            .values()
            .filter(|p| p.team == player.team)
            .map(|p| p.id)
            .collect();

        let team_info = ai::TeamInfo {
            team_id: player.team,
            player_count: team_members.len(),
            mech_count: game
                .mechs
                .values()
                .filter(|m| m.team == player.team)
                .count(),
            total_resources: game
                .mechs
                .values()
                .find(|m| m.team == player.team)
                .map(|m| m.resource_inventory.clone())
                .unwrap_or_default(),
        };

        GameView {
            tick: game.tick_count,
            players,
            mechs,
            resources,
            projectiles,
            team_info,
        }
    }

    /// Convert AI commands to game messages
    fn process_ai_commands(&self, commands: Vec<AICommand>) -> Vec<ServerMessage> {
        let messages = Vec::new();

        for command in commands {
            match command {
                AICommand::Move {
                    player_id,
                    movement,
                } => {
                    // The Move command doesn't generate a message directly
                    // Movement is handled in the update method
                }
                AICommand::PressButton {
                    player_id,
                    button_index,
                } => {
                    // This would need to be converted to appropriate station action
                    // For now, simplified to pressing primary button
                }
                AICommand::ExitMech { player_id } => {
                    // Would need to handle exiting mech
                    // For now, log the action
                    log::debug!("AI {player_id} wants to exit mech");
                }
                AICommand::EngineControl {
                    player_id,
                    movement,
                } => {
                    // Would need to handle engine control for mechs
                    log::debug!("AI {player_id} wants to control engine: {movement:?}");
                }
            }
        }

        messages
    }
}

impl GameSystem for AISystem {
    fn update(&mut self, game: &mut Game, delta_time: f32) -> Vec<ServerMessage> {
        let mut all_messages = Vec::new();

        // Check if we have any AI players
        if self.ai_players.is_empty() {
            return all_messages;
        }

        // Create a game view for the first AI (the AIManager expects a single view)
        // In a more sophisticated implementation, we might create per-team views
        let first_ai_id = self.ai_players.keys().next().copied();
        if let Some(ai_id) = first_ai_id {
            if game.players.contains_key(&ai_id) {
                let game_view = self.create_game_view(game, ai_id);

                // Update all AIs and get commands
                let commands = self.ai_manager.update(&game_view, delta_time);

                // Process commands into game actions
                for command in commands {
                    match command {
                        AICommand::Move {
                            player_id,
                            movement,
                        } => {
                            if let Some(player) = game.players.get_mut(&player_id) {
                                // Apply movement only if player is outside world
                                if let PlayerLocation::OutsideWorld(mut position) = player.location
                                {
                                    // Update position based on movement
                                    position.x += movement.0 * delta_time * 100.0;
                                    position.y += movement.1 * delta_time * 100.0;

                                    // Keep in bounds
                                    position.x = position
                                        .x
                                        .max(0.0)
                                        .min((ARENA_WIDTH_TILES as f32) * TILE_SIZE);
                                    position.y = position
                                        .y
                                        .max(0.0)
                                        .min((ARENA_HEIGHT_TILES as f32) * TILE_SIZE);

                                    // Update player location
                                    player.location = PlayerLocation::OutsideWorld(position);

                                    all_messages.push(ServerMessage::PlayerMoved {
                                        player_id,
                                        location: player.location,
                                    });
                                }
                            }
                        }
                        AICommand::PressButton {
                            player_id,
                            button_index,
                        } => {
                            if let Some(player) = game.players.get(&player_id) {
                                if let Some(station_id) = player.operating_station {
                                    // Find which mech contains this station
                                    let mech_station_info =
                                        game.mechs.iter_mut().find_map(|(mech_id, mech)| {
                                            mech.stations
                                                .get(&station_id)
                                                .map(|station| (*mech_id, station.station_type))
                                        });

                                    if let Some((mech_id, station_type)) = mech_station_info {
                                        // Simulate button press based on station type
                                        match station_type {
                                            StationType::WeaponLaser => {
                                                if let Some(_mech) = game.mechs.get(&mech_id) {
                                                    // Fire laser weapon
                                                    // For now, log the action
                                                    log::debug!(
                                                        "AI {player_id} pressed button {button_index} on laser station"
                                                    );
                                                }
                                            }
                                            StationType::WeaponProjectile => {
                                                if let Some(_mech) = game.mechs.get(&mech_id) {
                                                    // Fire projectile weapon
                                                    // For now, log the action
                                                    log::debug!("AI {player_id} pressed button {button_index} on projectile station");
                                                }
                                            }
                                            StationType::Shield => {
                                                // Activate shield
                                                // For now, log the action
                                                log::debug!(
                                                    "AI {player_id} pressed button {button_index} on shield station"
                                                );
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        all_messages
    }

    fn name(&self) -> &'static str {
        "AISystem"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
