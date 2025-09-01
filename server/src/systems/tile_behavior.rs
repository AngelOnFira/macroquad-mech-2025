use crate::entity_storage::EntityStorage;
use crate::game::Game;
use crate::game::Player;
use crate::systems::GameSystem;
use shared::{
    components::*,
    coordinates::MechDoorPositions,
    tile_entity::{TileEvent, TileMap},
    types::{TilePos, WorldPos},
    PlayerLocation, ResourceType, ServerMessage, TeamId, FLOOR_HEIGHT_TILES, FLOOR_WIDTH_TILES,
    MECH_SIZE_TILES, TILE_SIZE,
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct TileBehaviorSystem {
    pub event_queue: Vec<TileEvent>,
    pub time_elapsed: f32,
}

impl TileBehaviorSystem {
    pub fn new() -> Self {
        Self {
            event_queue: Vec::new(),
            time_elapsed: 0.0,
        }
    }

    pub fn process_behaviors(&mut self, delta_time: f32, game: &Game) -> Vec<TileEvent> {
        self.time_elapsed += delta_time;
        self.event_queue.clear();

        // Process proximity triggers
        self.process_proximity_triggers(&game.entity_storage, &game.players);

        // Process resource pickups
        self.process_resource_pickups(&game.entity_storage, &game.players);

        // Process mech entrances
        self.process_mech_entrances(&game.entity_storage, &game.players, &game.mechs);

        // Process auto interactions
        self.process_auto_interactions(&game.entity_storage, &game.players);

        // Return collected events
        std::mem::take(&mut self.event_queue)
    }

    fn process_proximity_triggers(
        &mut self,
        entities: &EntityStorage,
        players: &HashMap<Uuid, Player>,
    ) {
        // Check all entities with proximity triggers
        for (entity_id, trigger) in &entities.proximity_triggers {
            // Get entity position
            let entity_pos = match entities.positions.get(entity_id) {
                Some(pos) => pos,
                None => continue,
            };

            // Check each player
            for (player_id, player) in players {
                let player_pos = get_player_world_pos(&player.location);
                let distance = calculate_distance(&entity_pos.world, &player_pos);

                if distance <= trigger.range {
                    // Check team restrictions
                    if let Some(teams) = &trigger.trigger_for_teams {
                        if !teams.contains(&player.team) {
                            continue;
                        }
                    }

                    // Check cooldown
                    if let Some(last_trigger) = trigger.last_triggered.get(player_id) {
                        if self.time_elapsed - last_trigger < trigger.cooldown {
                            continue;
                        }
                    }

                    // Generate proximity event
                    self.event_queue.push(TileEvent::ProximityTriggered {
                        entity: *entity_id,
                        actor: *player_id,
                        distance,
                    });
                }
            }
        }
    }

    fn process_resource_pickups(
        &mut self,
        entities: &EntityStorage,
        players: &HashMap<Uuid, Player>,
    ) {
        // Check all entities with resource pickup components
        for (entity_id, pickup) in &entities.resource_pickups {
            // Skip if not auto-pickup
            if !pickup.auto_pickup {
                continue;
            }

            // Get entity position
            let entity_pos = match entities.positions.get(entity_id) {
                Some(pos) => pos,
                None => continue,
            };

            // Check each player
            for (player_id, player) in players {
                // Skip if player already carrying something
                if player.carrying_resource.is_some() {
                    continue;
                }

                let player_pos = get_player_world_pos(&player.location);
                let distance = calculate_distance(&entity_pos.world, &player_pos);

                if distance <= pickup.pickup_range {
                    // Generate pickup event
                    self.event_queue.push(TileEvent::ResourcePickedUp {
                        resource_entity: *entity_id,
                        actor: *player_id,
                        resource_type: pickup.resource_type,
                    });
                }
            }
        }
    }

    fn process_mech_entrances(
        &mut self,
        entities: &EntityStorage,
        players: &HashMap<Uuid, Player>,
        mechs: &HashMap<Uuid, crate::game::Mech>,
    ) {
        // Check all entities with mech entrance components
        for (entity_id, entrance) in &entities.mech_entrances {
            // Get entity position
            let entity_pos = match entities.positions.get(entity_id) {
                Some(pos) => pos,
                None => continue,
            };

            // Check each player
            for (player_id, player) in players {
                // Skip if player already in a mech
                if matches!(player.location, PlayerLocation::InsideMech { .. }) {
                    continue;
                }

                // Check team restriction
                if let Some(team) = entrance.team_restricted {
                    if player.team != team {
                        continue;
                    }
                }

                // Check if mech exists and belongs to player's team
                if let Some(mech) = mechs.get(&entrance.mech_id) {
                    if mech.team != player.team {
                        continue;
                    }
                }

                let player_pos = get_player_world_pos(&player.location);
                let distance = calculate_distance(&entity_pos.world, &player_pos);

                // Default entrance range
                if distance <= 1.0 * 16.0 {
                    // 1 tile
                    self.event_queue.push(TileEvent::MechEntered {
                        mech_id: entrance.mech_id,
                        actor: *player_id,
                        floor: entrance.target_floor,
                    });
                }
            }
        }
    }

    fn process_auto_interactions(
        &mut self,
        entities: &EntityStorage,
        players: &HashMap<Uuid, Player>,
    ) {
        // Check all entities with auto-interact components
        for (entity_id, auto) in &entities.auto_interacts {
            // Get entity position
            let entity_pos = match entities.positions.get(entity_id) {
                Some(pos) => pos,
                None => continue,
            };

            // Check each player
            for (player_id, player) in players {
                // Check conditions
                let conditions_met = auto.conditions.iter().all(|cond| match cond {
                    InteractionCondition::PlayerNotCarrying => player.carrying_resource.is_none(),
                    InteractionCondition::PlayerCarrying(resource_type) => {
                        player.carrying_resource == Some(*resource_type)
                    }
                    InteractionCondition::PlayerOnTeam(team) => player.team == *team,
                    InteractionCondition::PlayerOperatingStation(operating) => {
                        player.operating_station.is_some() == *operating
                    }
                });

                if !conditions_met {
                    continue;
                }

                let player_pos = get_player_world_pos(&player.location);
                let distance = calculate_distance(&entity_pos.world, &player_pos);

                if distance <= auto.range {
                    self.event_queue.push(TileEvent::AutoInteractionTriggered {
                        entity: *entity_id,
                        actor: *player_id,
                        action: auto.interaction_type.clone(),
                    });
                }
            }
        }
    }
}

impl GameSystem for TileBehaviorSystem {
    fn update(&mut self, game: &mut Game, delta_time: f32) -> Vec<ServerMessage> {
        // Process tile behaviors and get events
        let events = self.process_behaviors(delta_time, game);

        // Process each event and generate appropriate server messages
        let mut messages = Vec::new();

        for event in events {
            match event {
                TileEvent::ResourcePickedUp {
                    resource_entity,
                    actor,
                    resource_type,
                } => {
                    // Update game state
                    if let Some(player) = game.players.get_mut(&actor) {
                        if player.carrying_resource.is_none() {
                            player.carrying_resource = Some(resource_type);
                            // Remove resource from world
                            game.remove_resource(resource_entity);

                            messages.push(ServerMessage::PlayerPickedUpResource {
                                player_id: actor,
                                resource_id: resource_entity,
                                resource_type,
                            });
                        }
                    }
                }
                TileEvent::MechEntered {
                    mech_id,
                    actor,
                    floor,
                } => {
                    // Update player location
                    if let Some(player) = game.players.get_mut(&actor) {
                        // Calculate entry position based on where player entered from
                        // Players enter near the bottom of floor 0, in the center
                        let entry_pos = if let PlayerLocation::OutsideWorld(current_pos) =
                            player.location
                        {
                            // Get player's tile position to determine which door they used
                            let tile_pos = current_pos.to_tile_pos();

                            // Find mech position and use door abstraction
                            if let Some(mech) = game.mechs.get(&mech_id) {
                                let doors = MechDoorPositions::from_mech_position(mech.position);
                                doors.get_entry_position(tile_pos)
                            } else {
                                // Fallback if mech not found - use center entry position
                                let fallback_doors =
                                    MechDoorPositions::from_mech_position(TilePos::new(0, 0));
                                fallback_doors.get_entry_position(TilePos::new(0, 0))
                            }
                        } else {
                            // Fallback for unexpected location state - use center entry position
                            let fallback_doors =
                                MechDoorPositions::from_mech_position(TilePos::new(0, 0));
                            fallback_doors.get_entry_position(TilePos::new(0, 0))
                        };

                        player.location = PlayerLocation::InsideMech {
                            mech_id,
                            floor,
                            pos: entry_pos,
                        };

                        messages.push(ServerMessage::PlayerMoved {
                            player_id: actor,
                            location: player.location,
                        });
                    }
                }
                TileEvent::ProximityTriggered { entity, actor, .. } => {
                    // Could send a notification to the player
                    // For now, just log it
                    log::debug!(
                        "Proximity trigger: entity {:?} triggered by {:?}",
                        entity,
                        actor
                    );
                }
                TileEvent::AutoInteractionTriggered {
                    entity,
                    actor,
                    action,
                } => {
                    log::debug!(
                        "Auto interaction: {:?} on entity {:?} by {:?}",
                        action,
                        entity,
                        actor
                    );
                    // Handle based on action type
                    match action {
                        AutoInteractionType::DropResource => {
                            if let Some(player) = game.players.get_mut(&actor) {
                                if let Some(resource_type) = player.carrying_resource {
                                    // Get player's current location
                                    let (mech_to_deposit, tile_pos) = match player.location {
                                        PlayerLocation::InsideMech { mech_id, pos, .. } => {
                                            // Player is inside a mech - deposit to that mech
                                            (Some(mech_id), pos.to_tile_pos())
                                        }
                                        PlayerLocation::OutsideWorld(pos) => {
                                            // Player is outside - shouldn't happen with new cargo bay system
                                            // but keep as fallback
                                            let tile_pos = pos.to_tile_pos();
                                            let mut found_mech = None;

                                            for (mech_id, mech) in &game.mechs {
                                                if mech.team == player.team {
                                                    let dx = (mech.position.x - tile_pos.x).abs();
                                                    let dy = (mech.position.y - tile_pos.y).abs();
                                                    if dx < 5 && dy < 5 {
                                                        found_mech = Some(*mech_id);
                                                        break;
                                                    }
                                                }
                                            }
                                            (found_mech, tile_pos)
                                        }
                                    };

                                    // Deposit resource to the mech
                                    if let Some(mech_id) = mech_to_deposit {
                                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                                            if mech.team == player.team {
                                                *mech
                                                    .resource_inventory
                                                    .entry(resource_type)
                                                    .or_insert(0) += 1;
                                                player.carrying_resource = None;

                                                messages.push(
                                                    ServerMessage::PlayerDroppedResource {
                                                        player_id: actor,
                                                        resource_type,
                                                        position: tile_pos,
                                                    },
                                                );

                                                log::info!(
                                                    "Player {} deposited {:?} to mech cargo bay",
                                                    actor,
                                                    resource_type
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Update proximity trigger cooldowns
        for (_, trigger) in &mut game.entity_storage.proximity_triggers {
            // Clean up old cooldowns
            trigger
                .last_triggered
                .retain(|_, time| self.time_elapsed - *time < trigger.cooldown * 2.0);
        }

        messages
    }

    fn name(&self) -> &'static str {
        "TileBehaviorSystem"
    }

    fn should_update(&self, _game: &Game) -> bool {
        true // Always update to check proximities
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// Helper functions
fn get_player_world_pos(location: &PlayerLocation) -> WorldPos {
    match location {
        PlayerLocation::OutsideWorld(pos) => *pos,
        PlayerLocation::InsideMech { pos, .. } => *pos,
    }
}

fn calculate_distance(pos1: &WorldPos, pos2: &WorldPos) -> f32 {
    let dx = pos1.x - pos2.x;
    let dy = pos1.y - pos2.y;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{PlayerLocation, TeamId};

    #[test]
    fn test_proximity_trigger() {
        let mut system = TileBehaviorSystem::new();
        let mut entities = EntityStorage::new();
        let mut players = HashMap::new();

        // Create entity with proximity trigger
        let entity_id = Uuid::new_v4();
        entities.positions.insert(
            entity_id,
            Position {
                tile: TilePos::new(5, 5),
                world: WorldPos::new(80.0, 80.0),
                floor: None,
                mech_id: None,
            },
        );
        entities.proximity_triggers.insert(
            entity_id,
            ProximityTrigger {
                range: 32.0,
                trigger_for_teams: None,
                cooldown: 1.0,
                last_triggered: HashMap::new(),
            },
        );

        // Create player near entity
        let player_id = Uuid::new_v4();
        players.insert(
            player_id,
            Player {
                id: player_id,
                name: "Test".to_string(),
                team: TeamId::Red,
                location: PlayerLocation::OutsideWorld(WorldPos::new(85.0, 85.0)),
                carrying_resource: None,
                operating_station: None,
            },
        );

        // Process triggers
        system.process_proximity_triggers(&entities, &players);

        // Check event was generated
        assert_eq!(system.event_queue.len(), 1);
        match &system.event_queue[0] {
            TileEvent::ProximityTriggered { entity, actor, .. } => {
                assert_eq!(*entity, entity_id);
                assert_eq!(*actor, player_id);
            }
            _ => panic!("Wrong event type"),
        }
    }
}
