use macroquad::prelude::*;
use shared::{types::*, constants::*};
use crate::game_state::*;
use super::utils::*;

pub fn render_world_view(game_state: &GameState, cam_x: f32, cam_y: f32) {
    render_grass_background(cam_x, cam_y);
    render_arena_boundaries(cam_x, cam_y);
    render_mechs(game_state, cam_x, cam_y);
    render_resources(game_state, cam_x, cam_y);
    render_projectiles(game_state, cam_x, cam_y);
    render_players_in_world(game_state, cam_x, cam_y);
}

fn render_grass_background(cam_x: f32, cam_y: f32) {
    let grass_color = Color::new(0.2, 0.6, 0.2, 1.0);
    let grass_tile_size = TILE_SIZE * 2.0;
    
    // Calculate visible area with some padding
    let screen_w = screen_width();
    let screen_h = screen_height();
    let start_x = ((-cam_x / grass_tile_size).floor() * grass_tile_size) as i32;
    let start_y = ((-cam_y / grass_tile_size).floor() * grass_tile_size) as i32;
    let end_x = start_x + (screen_w / grass_tile_size) as i32 + 2;
    let end_y = start_y + (screen_h / grass_tile_size) as i32 + 2;
    
    for ty in start_y..end_y {
        for tx in start_x..end_x {
            let x = tx as f32 * grass_tile_size;
            let y = ty as f32 * grass_tile_size;
            
            // Vary grass color slightly for texture
            let variation = ((tx * 17 + ty * 13) % 20) as f32 / 200.0 - 0.05;
            let tile_color = Color::new(
                (grass_color.r + variation).clamp(0.0, 1.0),
                (grass_color.g + variation).clamp(0.0, 1.0),
                (grass_color.b + variation).clamp(0.0, 1.0),
                1.0
            );
            
            draw_rectangle(
                cam_x + x,
                cam_y + y,
                grass_tile_size,
                grass_tile_size,
                tile_color
            );
        }
    }
}

fn render_arena_boundaries(cam_x: f32, cam_y: f32) {
    let arena_width = ARENA_WIDTH_TILES as f32 * TILE_SIZE;
    let arena_height = ARENA_HEIGHT_TILES as f32 * TILE_SIZE;
    
    draw_rectangle_lines(
        cam_x,
        cam_y,
        arena_width,
        arena_height,
        3.0,
        GRAY
    );
}

fn render_mechs(game_state: &GameState, cam_x: f32, cam_y: f32) {
    for mech in game_state.mechs.values() {
        let mech_size = MECH_SIZE_TILES as f32 * TILE_SIZE;
        let color = get_team_color(mech.team);
        
        let mech_x = cam_x + mech.position.x as f32 * TILE_SIZE;
        let mech_y = cam_y + mech.position.y as f32 * TILE_SIZE;
        
        // Main body
        draw_rectangle(mech_x, mech_y, mech_size, mech_size, color);
        draw_rectangle_lines(mech_x, mech_y, mech_size, mech_size, 2.0, WHITE);
        
        // Entry door
        render_mech_entry_door(mech_x, mech_y, mech_size, color);
        
        // Resource drop-off zone
        render_resource_dropoff_zone(mech_x, mech_y, mech_size, color);
    }
}

fn render_mech_entry_door(mech_x: f32, mech_y: f32, mech_size: f32, mech_color: Color) {
    let door_width = TILE_SIZE * 1.5;
    let door_height = TILE_SIZE * 0.8;
    let door_x = mech_x + (mech_size - door_width) / 2.0;
    let door_y = mech_y + mech_size - door_height;
    
    // Door background (darker than mech)
    draw_rectangle(
        door_x,
        door_y,
        door_width,
        door_height,
        Color::new(mech_color.r * 0.3, mech_color.g * 0.3, mech_color.b * 0.3, 1.0)
    );
    
    draw_rectangle_lines(door_x, door_y, door_width, door_height, 2.0, WHITE);
    
    draw_text(
        "ENTER",
        door_x + door_width / 2.0 - 20.0,
        door_y + door_height / 2.0 + 6.0,
        14.0,
        WHITE
    );
}

fn render_resource_dropoff_zone(mech_x: f32, mech_y: f32, mech_size: f32, mech_color: Color) {
    let dropoff_width = TILE_SIZE * 2.0;
    let dropoff_height = TILE_SIZE * 2.0;
    let dropoff_x = mech_x + mech_size + TILE_SIZE * 0.5;
    let dropoff_y = mech_y + (mech_size - dropoff_height) / 2.0;
    
    // Drop-off zone background
    let dropoff_color = Color::new(mech_color.r * 0.5, mech_color.g * 0.5, mech_color.b * 0.5, 0.3);
    draw_rectangle(dropoff_x, dropoff_y, dropoff_width, dropoff_height, dropoff_color);
    
    // Dashed outline
    draw_dashed_rectangle_lines(dropoff_x, dropoff_y, dropoff_width, dropoff_height, 8.0, 4.0, 2.0, WHITE);
    
    // Text
    draw_text("DROP", dropoff_x + dropoff_width / 2.0 - 16.0, dropoff_y + dropoff_height / 2.0 - 8.0, 12.0, WHITE);
    draw_text("RESOURCES", dropoff_x + dropoff_width / 2.0 - 35.0, dropoff_y + dropoff_height / 2.0 + 8.0, 12.0, WHITE);
}

fn render_resources(game_state: &GameState, cam_x: f32, cam_y: f32) {
    for resource in &game_state.resources {
        let color = get_resource_color(resource.resource_type);
        
        draw_circle(
            cam_x + resource.position.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            cam_y + resource.position.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            TILE_SIZE / 3.0,
            color
        );
    }
}

fn render_projectiles(game_state: &GameState, cam_x: f32, cam_y: f32) {
    for projectile in &game_state.projectiles {
        draw_circle(
            cam_x + projectile.position.x,
            cam_y + projectile.position.y,
            5.0,
            YELLOW
        );
    }
}

fn render_players_in_world(game_state: &GameState, cam_x: f32, cam_y: f32) {
    for player in game_state.players.values() {
        if let PlayerLocation::OutsideWorld(pos) = player.location {
            let color = get_player_color(player.team);
            
            // Player body
            draw_circle(cam_x + pos.x, cam_y + pos.y, TILE_SIZE / 2.0, color);
            
            // Player name
            draw_text(
                &player.name,
                cam_x + pos.x - 20.0,
                cam_y + pos.y - TILE_SIZE - 5.0,
                16.0,
                WHITE
            );
            
            // Resource being carried
            if let Some(resource_type) = player.carrying_resource {
                let resource_color = get_resource_color(resource_type);
                draw_circle(
                    cam_x + pos.x + TILE_SIZE,
                    cam_y + pos.y,
                    TILE_SIZE / 4.0,
                    resource_color
                );
            }
        }
    }
}