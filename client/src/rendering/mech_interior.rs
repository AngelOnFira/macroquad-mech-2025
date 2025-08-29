use macroquad::prelude::*;
use shared::{types::*, constants::*, tile_entity::TileVisual};
use crate::game_state::*;
use super::utils::*;
use uuid::Uuid;

pub fn render_mech_interior(game_state: &GameState, mech: &MechState, current_floor: u8, cam_x: f32, cam_y: f32) {
    // The hybrid tile system sends all visible tiles through the same channel
    // We need to filter for tiles that belong to this mech interior
    // For now, render a basic floor grid as fallback
    for y in 0..FLOOR_HEIGHT_TILES {
        for x in 0..FLOOR_WIDTH_TILES {
            let tile_x = cam_x + x as f32 * TILE_SIZE;
            let tile_y = cam_y + y as f32 * TILE_SIZE;
            
            // Basic floor rendering
            if x == 0 || x == FLOOR_WIDTH_TILES - 1 || y == 0 || y == FLOOR_HEIGHT_TILES - 1 {
                // Wall
                draw_rectangle(tile_x, tile_y, TILE_SIZE, TILE_SIZE, LIGHTGRAY);
            } else {
                // Floor
                draw_rectangle(tile_x, tile_y, TILE_SIZE, TILE_SIZE, DARKGRAY);
                draw_rectangle_lines(tile_x, tile_y, TILE_SIZE, TILE_SIZE, 1.0, GRAY);
            }
        }
    }
    
    // TODO: Once server sends mech interior tiles via visible_tiles,
    // render them using the hybrid_tiles renderer
}


pub fn render_stations_on_floor(game_state: &GameState, mech_id: Uuid, floor: u8) {
    for station in game_state.stations.values() {
        if station.mech_id == mech_id && station.floor == floor {
            let x = station.position.x as f32 * TILE_SIZE;
            let y = station.position.y as f32 * TILE_SIZE - 5.0;
            
            let label = get_station_label(station.station_type);
            draw_text(label, x, y, 16.0, WHITE);
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
    for player in game_state.players.values() {
        if let PlayerLocation::InsideMech { mech_id: player_mech_id, floor: player_floor, pos } = player.location {
            if player_mech_id == mech_id && player_floor == floor {
                let color = get_player_color(player.team);
                
                draw_circle(
                    cam_x + pos.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                    cam_y + pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                    TILE_SIZE / 2.5,
                    color
                );
                
                // Draw player name
                draw_text(
                    &player.name,
                    cam_x + pos.x as f32 * TILE_SIZE - 20.0,
                    cam_y + pos.y as f32 * TILE_SIZE - 5.0,
                    14.0,
                    WHITE
                );
            }
        }
    }
}