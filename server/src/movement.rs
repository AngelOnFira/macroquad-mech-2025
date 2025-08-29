use tokio::sync::broadcast;
use uuid::Uuid;
use shared::*;
use shared::tile_entity::{TileContent, StaticTile, TileEvent, TransitionType};
use crate::game::Game;

/// Handle all player movement logic
pub async fn handle_player_movement(
    game: &mut Game,
    player_id: Uuid,
    movement: (f32, f32),
    tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    // Get player info we need
    let (current_location, _is_carrying_resource, player_team) = {
        if let Some(player) = game.players.get(&player_id) {
            (player.location, player.carrying_resource.is_some(), player.team)
        } else {
            return;
        }
    };

    match current_location {
        PlayerLocation::OutsideWorld(pos) => {
            handle_outside_world_movement(game, player_id, pos, movement, player_team, tx).await;
        }
        PlayerLocation::InsideMech { mech_id, floor, pos } => {
            handle_inside_mech_movement(game, player_id, mech_id, floor, pos, movement, tx).await;
        }
    }
}

/// Handle movement when player is outside in the world
async fn handle_outside_world_movement(
    game: &mut Game,
    player_id: Uuid,
    current_pos: WorldPos,
    movement: (f32, f32),
    player_team: TeamId,
    tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    let new_pos = calculate_new_world_position(current_pos, movement);
    
    if !is_world_position_valid(new_pos) {
        return;
    }
    
    // Get the tile at the new position
    let tile_pos = new_pos.to_tile_pos();
    if let Some(tile_content) = game.tile_map.get_world_tile(tile_pos) {
        let is_walkable = match tile_content {
            TileContent::Empty => true,
            TileContent::Static(static_tile) => static_tile.is_walkable(),
            TileContent::Entity(_) => {
                // TODO: Check entity properties
                false
            }
        };
        
        // Check if tile is walkable
        if !is_walkable {
            return;
        }
        
        // Check for mech collision unless on special tiles
        let is_transition = matches!(tile_content, TileContent::Static(StaticTile::TransitionZone { .. }));
        if !is_transition && has_mech_collision(game, new_pos) {
            return;
        }
        
        // Handle tile interactions
        if let TileContent::Static(static_tile) = tile_content {
            if let Some(event) = static_tile.on_enter(player_id) {
                match event {
                    TileEvent::BeginTransition { actor: _, zone_id: _, transition_type } => {
                        match transition_type {
                            TransitionType::MechEntrance { stage } => {
                                if stage == 1 {
                                    // Find which mech we're entering
                                    for (mech_id, mech) in &game.mechs {
                                        let door_y = mech.position.y + MECH_SIZE_TILES - 1;
                                        let door_x1 = mech.position.x + (MECH_SIZE_TILES / 2) - 1;
                                        let door_x2 = mech.position.x + (MECH_SIZE_TILES / 2);
                                        
                                        if tile_pos.y == door_y && (tile_pos.x == door_x1 || tile_pos.x == door_x2) {
                                            if mech.team == player_team {
                                                let entry_location = PlayerLocation::InsideMech {
                                                    mech_id: *mech_id,
                                                    floor: 0,
                                                    pos: WorldPos::new(1.5 * TILE_SIZE, (FLOOR_HEIGHT_TILES as f32 / 2.0) * TILE_SIZE),
                                                };
                                                update_player_location(game, player_id, entry_location, tx).await;
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {} // Other transition types
                        }
                    }
                    _ => {} // Other events
                }
            }
        }
        
        // Handle cargo floor resource drop-off
        if matches!(tile_content, TileContent::Static(StaticTile::CargoFloor { .. })) {
            // Check if this is on top of a mech
            let mut found_mech = None;
            for (mech_id, mech) in &game.mechs {
                let dropoff_x = mech.position.x + (MECH_SIZE_TILES / 2) - 1;
                let dropoff_y = mech.position.y;
                
                if tile_pos.x >= dropoff_x && tile_pos.x < dropoff_x + 3 &&
                   tile_pos.y >= dropoff_y && tile_pos.y < dropoff_y + 3 {
                    found_mech = Some((*mech_id, mech.team));
                    break;
                }
            }
            
            if let Some((mech_id, mech_team)) = found_mech {
                if let Some(resource_type) = game.players.get(&player_id)
                    .and_then(|p| p.carrying_resource) {
                    
                    if mech_team == player_team {
                        // Add resource to mech inventory
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            *mech.resource_inventory.entry(resource_type).or_insert(0) += 1;
                        }
                        
                        // Remove from player
                        if let Some(player) = game.players.get_mut(&player_id) {
                            player.carrying_resource = None;
                        }
                        
                        // Notify clients
                        let _ = tx.send((Uuid::nil(), ServerMessage::PlayerDroppedResource {
                            player_id,
                            resource_type,
                            position: tile_pos,
                        }));
                        
                        log::info!("Player {} dropped {} on mech", player_id, resource_type as u8);
                    }
                }
            }
        }
    } else {
        // No tile at position, can't move there
        return;
    }
    
    // Normal movement
    let new_location = PlayerLocation::OutsideWorld(new_pos);
    update_player_location(game, player_id, new_location, tx).await;
}

/// Handle movement when player is inside a mech
async fn handle_inside_mech_movement(
    game: &mut Game,
    player_id: Uuid,
    mech_id: Uuid,
    current_floor: u8,
    current_pos: WorldPos,
    movement: (f32, f32),
    tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    let new_pos = calculate_new_world_position(current_pos, movement);
    
    if !is_mech_position_valid(new_pos) {
        return;
    }
    
    let tile_pos = new_pos.to_tile_pos();
    if !is_tile_position_valid_for_floor(tile_pos) {
        return;
    }
    
    let mut floor = current_floor;
    
    let new_location = PlayerLocation::InsideMech { 
        mech_id, 
        floor, 
        pos: new_pos 
    };
    update_player_location(game, player_id, new_location, tx).await;
}

/// Calculate new position based on movement input
fn calculate_new_world_position(current_pos: WorldPos, movement: (f32, f32)) -> WorldPos {
    let delta_time = CONTINUOUS_MOVEMENT_DELTA;
    WorldPos::new(
        current_pos.x + movement.0 * PLAYER_MOVE_SPEED * TILE_SIZE * delta_time,
        current_pos.y + movement.1 * PLAYER_MOVE_SPEED * TILE_SIZE * delta_time,
    )
}

/// Check if a world position is within valid bounds
fn is_world_position_valid(pos: WorldPos) -> bool {
    pos.x >= 0.0 && pos.x < (ARENA_WIDTH_TILES as f32 * TILE_SIZE) &&
    pos.y >= 0.0 && pos.y < (ARENA_HEIGHT_TILES as f32 * TILE_SIZE)
}

/// Check if a mech interior position is within valid bounds
fn is_mech_position_valid(pos: WorldPos) -> bool {
    pos.x >= 0.0 && pos.x < (FLOOR_WIDTH_TILES as f32 * TILE_SIZE) &&
    pos.y >= 0.0 && pos.y < (FLOOR_HEIGHT_TILES as f32 * TILE_SIZE)
}

/// Check if tile position is valid for a mech floor
fn is_tile_position_valid_for_floor(tile_pos: TilePos) -> bool {
    tile_pos.x >= 0 && tile_pos.x < FLOOR_WIDTH_TILES &&
    tile_pos.y >= 0 && tile_pos.y < FLOOR_HEIGHT_TILES
}

/// Check if a tile type is walkable
fn is_tile_walkable(tile_content: &TileContent) -> bool {
    match tile_content {
        TileContent::Empty => true,
        TileContent::Static(static_tile) => static_tile.is_walkable(),
        TileContent::Entity(_) => false, // Entities block movement
    }
}

/// Check for collision with mechs in the world
fn has_mech_collision(game: &Game, pos: WorldPos) -> bool {
    let player_tile = pos.to_tile_pos();
    for mech in game.mechs.values() {
        // Mech occupies a 10x10 tile area
        if player_tile.x >= mech.position.x && player_tile.x < mech.position.x + MECH_SIZE_TILES &&
           player_tile.y >= mech.position.y && player_tile.y < mech.position.y + MECH_SIZE_TILES {
            return true;
        }
    }
    false
}

/// Check if player position allows entry into a team mech
fn check_mech_entry(game: &Game, pos: WorldPos, player_team: TeamId) -> Option<PlayerLocation> {
    for mech in game.mechs.values() {
        if mech.team == player_team {
            // Check if player walked into the mech door (middle of left side)
            let door_x = mech.position.x as f32 * TILE_SIZE;
            let door_y = (mech.position.y as f32 + MECH_SIZE_TILES as f32 / 2.0) * TILE_SIZE;
            let door_pos = WorldPos::new(door_x, door_y);
            
            if pos.distance_to(door_pos) < TILE_SIZE * MECH_DOOR_ENTRY_DISTANCE {
                return Some(PlayerLocation::InsideMech {
                    mech_id: mech.id,
                    floor: 0,
                    pos: WorldPos::new(1.5 * TILE_SIZE, (FLOOR_HEIGHT_TILES as f32 / 2.0) * TILE_SIZE),
                });
            }
        }
    }
    None
}


/// Handle ladder interaction for floor changes
fn handle_ladder_interaction(
    _current_pos: WorldPos,
    new_pos: WorldPos,
    movement: (f32, f32),
    current_floor: u8,
) -> u8 {
    let tile_pos = new_pos.to_tile_pos();
    let tile_center = WorldPos::new(
        (tile_pos.x as f32 + 0.5) * TILE_SIZE,
        (tile_pos.y as f32 + 0.5) * TILE_SIZE
    );
    
    // Only change floor if player is close to ladder center
    if new_pos.distance_to(tile_center) < TILE_SIZE * LADDER_INTERACTION_DISTANCE {
        if movement.1 < -0.5 && current_floor > 0 {
            current_floor - 1 // Up
        } else if movement.1 > 0.5 && current_floor < (MECH_FLOORS - 1) as u8 {
            current_floor + 1 // Down
        } else {
            current_floor
        }
    } else {
        current_floor
    }
}

/// Update player location and notify clients
async fn update_player_location(
    game: &mut Game,
    player_id: Uuid,
    new_location: PlayerLocation,
    tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    if let Some(player) = game.players.get_mut(&player_id) {
        player.location = new_location;
        let _ = tx.send((Uuid::nil(), ServerMessage::PlayerMoved {
            player_id,
            location: new_location,
        }));
    }
}