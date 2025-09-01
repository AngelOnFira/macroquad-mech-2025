use macroquad::prelude::*;
use shared::{types::*, constants::*, tile_entity::TileVisual, MechInteriorCoordinates};
use crate::game_state::*;
use crate::vision::{ClientVisionSystem, FogOfWarRenderer};
use super::utils::*;
use uuid::Uuid;

pub fn render_mech_interior(game_state: &GameState, mech: &MechState, current_floor: u8, cam_x: f32, cam_y: f32) {
    render_mech_interior_with_vision(game_state, mech, current_floor, cam_x, cam_y, None);
}

pub fn render_mech_interior_with_vision(
    game_state: &GameState, 
    mech: &MechState, 
    current_floor: u8, 
    cam_x: f32, 
    cam_y: f32, 
    vision_system: Option<&ClientVisionSystem>
) {
    // Render the mech interior using world coordinate mapping
    // This allows mech interiors to be seen from the outside through windows/doors
    
    for y in 0..FLOOR_HEIGHT_TILES {
        for x in 0..FLOOR_WIDTH_TILES {
            let interior_pos = TilePos::new(x, y);
            let world_pos = MechInteriorCoordinates::interior_to_world(mech.position, current_floor, interior_pos);
            let world_coords = world_pos.to_world();
            
            let tile_x = cam_x + world_coords.x;
            let tile_y = cam_y + world_coords.y;
            
            // Check if this interior tile is visible
            let mut visibility = 1.0;
            if let Some(vision) = vision_system {
                visibility = vision.get_interior_visibility(mech.id, current_floor, interior_pos);
                if visibility < 0.05 {
                    continue; // Don't render invisible interior tiles
                }
            }
            
            // Basic floor rendering (will be replaced by server tiles eventually)
            let mut base_color = if x == 0 || x == FLOOR_WIDTH_TILES - 1 || y == 0 || y == FLOOR_HEIGHT_TILES - 1 {
                // Wall
                LIGHTGRAY
            } else {
                // Floor
                DARKGRAY
            };
            
            // Apply fog of war
            if let Some(_vision) = vision_system {
                base_color = FogOfWarRenderer::apply_fog_to_color(base_color, visibility);
            }
            
            draw_rectangle(tile_x, tile_y, TILE_SIZE, TILE_SIZE, base_color);
            
            // Grid lines for floors
            if !(x == 0 || x == FLOOR_WIDTH_TILES - 1 || y == 0 || y == FLOOR_HEIGHT_TILES - 1) {
                let mut grid_color = GRAY;
                if let Some(_vision) = vision_system {
                    grid_color = FogOfWarRenderer::apply_fog_to_color(grid_color, visibility);
                }
                draw_rectangle_lines(tile_x, tile_y, TILE_SIZE, TILE_SIZE, 1.0, grid_color);
            }
        }
    }
    
    // Check for interior tiles from server data (hybrid system)
    for (tile_pos, tile_visual) in &game_state.visible_tiles {
        // Check if this world tile position maps to our mech interior
        if let Some((floor, interior_pos)) = MechInteriorCoordinates::world_to_interior(*tile_pos, mech.position) {
            if floor == current_floor {
                let tile_world = tile_pos.to_world();
                let tile_x = cam_x + tile_world.x;
                let tile_y = cam_y + tile_world.y;
                
                // Apply fog of war if available
                if let Some(vision) = vision_system {
                    let visibility = vision.get_visibility(*tile_pos);
                    if visibility > 0.05 {
                        super::hybrid_tiles::render_tile_visual_with_visibility(tile_visual, tile_x, tile_y, TILE_SIZE, visibility);
                    }
                } else {
                    super::hybrid_tiles::render_tile_visual(tile_visual, tile_x, tile_y, TILE_SIZE);
                }
            }
        }
    }
}


pub fn render_stations_on_floor(game_state: &GameState, mech_id: Uuid, floor: u8) {
    render_stations_on_floor_with_vision(game_state, mech_id, floor, 0.0, 0.0, None);
}

pub fn render_stations_on_floor_with_vision(
    game_state: &GameState, 
    mech_id: Uuid, 
    floor: u8,
    cam_x: f32,
    cam_y: f32, 
    vision_system: Option<&ClientVisionSystem>
) {
    for station in game_state.stations.values() {
        if station.mech_id == mech_id && station.floor == floor {
            // Map interior position to world coordinates
            let mech = game_state.mechs.get(&mech_id);
            if let Some(mech) = mech {
                let world_pos = MechInteriorCoordinates::interior_to_world(mech.position, floor, station.position);
                let world_coords = world_pos.to_world();
                let x = cam_x + world_coords.x;
                let y = cam_y + world_coords.y - 5.0;
                
                // Check visibility
                let mut text_color = WHITE;
                if let Some(vision) = vision_system {
                    let visibility = vision.get_interior_visibility(mech_id, floor, station.position);
                    if visibility < 0.1 {
                        continue; // Don't render invisible stations
                    }
                    text_color = FogOfWarRenderer::apply_fog_to_color(text_color, visibility);
                }
                
                let label = get_station_label(station.station_type);
                draw_text(label, x, y, 16.0, text_color);
            }
        }
    }
}

fn get_station_label(station_type: StationType) -> &'static str {
    match station_type {
        StationType::WeaponLaser => "LASER",
        StationType::WeaponProjectile => "GUN",
        StationType::Engine => "ENGINE",
        StationType::Shield => "SHIELD",
        StationType::Repair => "REPAIR",
        StationType::Electrical => "ELEC",
        StationType::Upgrade => "UPGRADE",
        StationType::Pilot => "PILOT",
    }
}

pub fn render_players_on_floor(game_state: &GameState, mech_id: Uuid, floor: u8, cam_x: f32, cam_y: f32) {
    render_players_on_floor_with_vision(game_state, mech_id, floor, cam_x, cam_y, None);
}

pub fn render_players_on_floor_with_vision(
    game_state: &GameState, 
    mech_id: Uuid, 
    floor: u8, 
    cam_x: f32, 
    cam_y: f32, 
    vision_system: Option<&ClientVisionSystem>
) {
    for player in game_state.players.values() {
        if let PlayerLocation::InsideMech { mech_id: player_mech_id, floor: player_floor, pos } = player.location {
            if player_mech_id == mech_id && player_floor == floor {
                // Map interior position to world coordinates
                if let Some(mech) = game_state.mechs.get(&mech_id) {
                    let interior_pos = pos.to_tile();
                    let world_pos = MechInteriorCoordinates::interior_to_world(mech.position, floor, interior_pos);
                    let world_coords = world_pos.to_world();
                    
                    let mut color = get_player_color(player.team);
                    let mut text_color = WHITE;
                    
                    // Check visibility
                    if let Some(vision) = vision_system {
                        let visibility = vision.get_interior_visibility(mech_id, floor, interior_pos);
                        if visibility < 0.1 {
                            continue; // Don't render invisible players
                        }
                        color = FogOfWarRenderer::apply_fog_to_color(color, visibility);
                        text_color = FogOfWarRenderer::apply_fog_to_color(text_color, visibility);
                    }
                    
                    draw_circle(
                        cam_x + world_coords.x + TILE_SIZE / 2.0,
                        cam_y + world_coords.y + TILE_SIZE / 2.0,
                        TILE_SIZE / 2.5,
                        color
                    );
                    
                    // Draw player name
                    draw_text(
                        &player.name,
                        cam_x + world_coords.x - 20.0,
                        cam_y + world_coords.y - 5.0,
                        14.0,
                        text_color
                    );
                }
            }
        }
    }
}