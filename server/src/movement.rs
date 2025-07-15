use tokio::sync::broadcast;
use uuid::Uuid;
use shared::*;
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
    
    if has_mech_collision(game, new_pos) {
        return;
    }
    
    // Check for automatic mech entry
    if let Some(mech_entry) = check_mech_entry(game, new_pos, player_team) {
        update_player_location(game, player_id, mech_entry, tx).await;
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
    // Get mech interior info
    let floor_layout = match get_mech_floor_layout(game, mech_id, current_floor) {
        Some(layout) => layout,
        None => return,
    };
    
    let new_pos = calculate_new_world_position(current_pos, movement);
    
    if !is_mech_position_valid(new_pos) {
        return;
    }
    
    let tile_pos = new_pos.to_tile_pos();
    if !is_tile_position_valid_for_floor(tile_pos) {
        return;
    }
    
    let tile = floor_layout.tiles[tile_pos.y as usize][tile_pos.x as usize];
    if !is_tile_walkable(tile) {
        return;
    }
    
    let mut floor = current_floor;
    
    // Handle ladder interaction
    if tile == TileType::Ladder {
        floor = handle_ladder_interaction(current_pos, new_pos, movement, current_floor);
    }
    
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
fn is_tile_walkable(tile: TileType) -> bool {
    tile != TileType::Wall && tile != TileType::Empty
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

/// Get floor layout for a mech
fn get_mech_floor_layout(game: &Game, mech_id: Uuid, floor: u8) -> Option<FloorLayout> {
    game.mechs.get(&mech_id)
        .and_then(|m| m.interior.floors.get(floor as usize))
        .cloned()
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