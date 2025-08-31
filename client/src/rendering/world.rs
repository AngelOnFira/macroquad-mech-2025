use macroquad::prelude::*;
use shared::{types::*, constants::*};
use crate::game_state::*;
use super::utils::*;

pub fn render_world_view(game_state: &GameState, cam_x: f32, cam_y: f32) {
    render_grass_background(cam_x, cam_y);
    render_arena_boundaries(cam_x, cam_y);
    render_mechs(game_state, cam_x, cam_y);
    render_world_tiles(game_state, cam_x, cam_y);
    render_resources(game_state, cam_x, cam_y);
    render_projectiles(game_state, cam_x, cam_y);
    render_players_in_world(game_state, cam_x, cam_y);
    
    // Debug info
    draw_text(
        &format!("Camera: ({:.1}, {:.1})", -cam_x, -cam_y),
        10.0,
        30.0,
        20.0,
        WHITE
    );
    draw_text(
        &format!("Mechs: {}, Players: {}", game_state.mechs.len(), game_state.players.len()),
        10.0,
        50.0,
        20.0,
        WHITE
    );
    if let Some(player_id) = game_state.player_id {
        if let PlayerLocation::OutsideWorld(pos) = game_state.player_location {
            draw_text(
                &format!("Player pos: ({:.1}, {:.1})", pos.x, pos.y),
                10.0,
                70.0,
                20.0,
                WHITE
            );
        }
    }
}

fn render_grass_background(cam_x: f32, cam_y: f32) {
    let grass_color = Color::new(0.2, 0.6, 0.2, 1.0);
    let grass_tile_size = TILE_SIZE * 2.0;
    
    // Calculate visible area with some padding
    let screen_w = screen_width();
    let screen_h = screen_height();
    let start_x = ((-cam_x / grass_tile_size).floor()) as i32 - 1;
    let start_y = ((-cam_y / grass_tile_size).floor()) as i32 - 1;
    let end_x = ((-cam_x + screen_w) / grass_tile_size).ceil() as i32 + 1;
    let end_y = ((-cam_y + screen_h) / grass_tile_size).ceil() as i32 + 1;
    
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
        
    }
}

fn render_world_tiles(game_state: &GameState, cam_x: f32, cam_y: f32) {
    // Render visible tiles sent from server
    for (tile_pos, tile_visual) in &game_state.visible_tiles {
        let tile_x = cam_x + tile_pos.x as f32 * TILE_SIZE;
        let tile_y = cam_y + tile_pos.y as f32 * TILE_SIZE;
        
        super::hybrid_tiles::render_tile_visual(tile_visual, tile_x, tile_y, TILE_SIZE);
    }
    
    // Still render mech-related tiles for now as fallback
    // TODO: Remove once server sends all tiles
    if game_state.visible_tiles.is_empty() {
        for mech in game_state.mechs.values() {
            let team_color = get_team_color(mech.team);
            
            // Render door tiles at bottom center of mech - 2 blocks wide
            let door_x1 = mech.position.x + (MECH_SIZE_TILES / 2) - 1;
            let door_x2 = mech.position.x + (MECH_SIZE_TILES / 2);
            let door_y = mech.position.y + MECH_SIZE_TILES - 1;
            render_door_tile(door_x1, door_y, team_color, cam_x, cam_y);
            render_door_tile(door_x2, door_y, team_color, cam_x, cam_y);
            
        }
    }
}

fn render_door_tile(x: i32, y: i32, team_color: Color, cam_x: f32, cam_y: f32) {
    let tile_x = cam_x + x as f32 * TILE_SIZE;
    let tile_y = cam_y + y as f32 * TILE_SIZE;
    
    // Door background (darker than mech)
    draw_rectangle(
        tile_x,
        tile_y,
        TILE_SIZE,
        TILE_SIZE,
        Color::new(team_color.r * 0.3, team_color.g * 0.3, team_color.b * 0.3, 1.0)
    );
    
    // Door outline
    draw_rectangle_lines(tile_x, tile_y, TILE_SIZE, TILE_SIZE, 2.0, WHITE);
    
    // Visual entry indicator - just a subtle arrow or pattern
    let arrow_color = Color::new(1.0, 1.0, 1.0, 0.5);
    draw_line(
        tile_x + TILE_SIZE / 2.0,
        tile_y + TILE_SIZE * 0.3,
        tile_x + TILE_SIZE / 2.0,
        tile_y + TILE_SIZE * 0.7,
        2.0,
        arrow_color
    );
    draw_line(
        tile_x + TILE_SIZE * 0.3,
        tile_y + TILE_SIZE * 0.5,
        tile_x + TILE_SIZE / 2.0,
        tile_y + TILE_SIZE * 0.7,
        2.0,
        arrow_color
    );
    draw_line(
        tile_x + TILE_SIZE * 0.7,
        tile_y + TILE_SIZE * 0.5,
        tile_x + TILE_SIZE / 2.0,
        tile_y + TILE_SIZE * 0.7,
        2.0,
        arrow_color
    );
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