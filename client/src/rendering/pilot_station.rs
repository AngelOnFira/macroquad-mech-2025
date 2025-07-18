use macroquad::prelude::*;
use shared::{types::*, constants::*};
use crate::game_state::GameState;
use super::utils::*;

const PILOT_WINDOW_WIDTH: f32 = 800.0;
const PILOT_WINDOW_HEIGHT: f32 = 600.0;
const MAP_ZOOM: f32 = 0.25; // Show mechs at 1/4 scale

pub fn render_pilot_station_window(game_state: &GameState) {
    if !game_state.ui_state.pilot_station_open {
        return;
    }

    let screen_width = screen_width();
    let screen_height = screen_height();
    
    // Calculate window position (centered)
    let window_x = (screen_width - PILOT_WINDOW_WIDTH) / 2.0;
    let window_y = (screen_height - PILOT_WINDOW_HEIGHT) / 2.0;
    
    // Draw window background
    draw_rectangle(
        window_x,
        window_y,
        PILOT_WINDOW_WIDTH,
        PILOT_WINDOW_HEIGHT,
        Color::new(0.1, 0.1, 0.1, 0.95)
    );
    
    // Draw window border
    draw_rectangle_lines(
        window_x,
        window_y,
        PILOT_WINDOW_WIDTH,
        PILOT_WINDOW_HEIGHT,
        3.0,
        GREEN
    );
    
    // Draw title bar
    draw_rectangle(
        window_x,
        window_y,
        PILOT_WINDOW_WIDTH,
        30.0,
        Color::new(0.2, 0.4, 0.2, 1.0)
    );
    
    draw_text(
        "PILOT STATION - MECH CONTROL",
        window_x + 10.0,
        window_y + 22.0,
        20.0,
        WHITE
    );
    
    // Draw close button
    let close_x = window_x + PILOT_WINDOW_WIDTH - 30.0;
    let close_y = window_y;
    draw_rectangle(close_x, close_y, 30.0, 30.0, Color::new(0.8, 0.2, 0.2, 1.0));
    draw_text("X", close_x + 10.0, close_y + 22.0, 20.0, WHITE);
    
    // Draw map area
    let map_x = window_x + 10.0;
    let map_y = window_y + 40.0;
    let map_width = PILOT_WINDOW_WIDTH - 20.0;
    let map_height = PILOT_WINDOW_HEIGHT - 100.0;
    
    // Map background
    draw_rectangle(
        map_x,
        map_y,
        map_width,
        map_height,
        Color::new(0.05, 0.05, 0.05, 1.0)
    );
    
    // Draw map border
    draw_rectangle_lines(map_x, map_y, map_width, map_height, 2.0, DARKGREEN);
    
    // Render area view
    render_area_view(game_state, map_x, map_y, map_width, map_height);
    
    // Draw control instructions at bottom
    let instruction_y = window_y + PILOT_WINDOW_HEIGHT - 50.0;
    draw_text(
        "WASD - Move Mech | ESC - Exit Pilot Mode",
        window_x + 10.0,
        instruction_y,
        16.0,
        LIGHTGRAY
    );
}

fn render_area_view(game_state: &GameState, map_x: f32, map_y: f32, map_width: f32, map_height: f32) {
    // Get our mech position if we're operating one
    let operating_mech = game_state.ui_state.operating_mech_id
        .and_then(|id| game_state.mechs.get(&id));
    
    if let Some(mech) = operating_mech {
        // Calculate the visible area centered on our mech
        let center_x = mech.world_position.x;
        let center_y = mech.world_position.y;
        
        // Calculate the world area visible in the map
        let world_visible_width = map_width / MAP_ZOOM;
        let world_visible_height = map_height / MAP_ZOOM;
        
        let world_left = center_x - world_visible_width / 2.0;
        let world_top = center_y - world_visible_height / 2.0;
        
        // Draw grid
        draw_grid(map_x, map_y, map_width, map_height, world_left, world_top, MAP_ZOOM);
        
        // Draw all mechs
        for (mech_id, other_mech) in &game_state.mechs {
            let screen_x = map_x + (other_mech.world_position.x - world_left) * MAP_ZOOM;
            let screen_y = map_y + (other_mech.world_position.y - world_top) * MAP_ZOOM;
            
            // Skip if outside visible area
            if screen_x < map_x || screen_x > map_x + map_width || 
               screen_y < map_y || screen_y > map_y + map_height {
                continue;
            }
            
            let mech_size = MECH_SIZE_TILES as f32 * TILE_SIZE * MAP_ZOOM;
            let color = if mech_id == &mech.id {
                // Our mech - highlight it
                Color::new(0.0, 1.0, 0.0, 0.8)
            } else {
                // Other mechs
                get_team_color(other_mech.team)
            };
            
            // Draw mech
            draw_rectangle(screen_x, screen_y, mech_size, mech_size, color);
            draw_rectangle_lines(screen_x, screen_y, mech_size, mech_size, 2.0, WHITE);
            
            // Draw mech ID/team indicator
            let label = if mech_id == &mech.id { "YOU" } else { 
                match other_mech.team {
                    TeamId::Red => "R",
                    TeamId::Blue => "B",
                }
            };
            draw_text(
                label,
                screen_x + mech_size / 2.0 - 10.0,
                screen_y + mech_size / 2.0 + 5.0,
                16.0,
                WHITE
            );
        }
        
        // Draw players in the world
        for player in game_state.players.values() {
            if let PlayerLocation::OutsideWorld(pos) = player.location {
                let screen_x = map_x + (pos.x - world_left) * MAP_ZOOM;
                let screen_y = map_y + (pos.y - world_top) * MAP_ZOOM;
                
                // Skip if outside visible area
                if screen_x < map_x || screen_x > map_x + map_width || 
                   screen_y < map_y || screen_y > map_y + map_height {
                    continue;
                }
                
                let player_color = get_player_color(player.team);
                draw_circle(screen_x, screen_y, 3.0, player_color);
            }
        }
        
        // Draw resources
        for resource in &game_state.resources {
            let resource_world_pos = resource.position.to_world_pos();
            let screen_x = map_x + (resource_world_pos.x - world_left) * MAP_ZOOM;
            let screen_y = map_y + (resource_world_pos.y - world_top) * MAP_ZOOM;
            
            // Skip if outside visible area
            if screen_x < map_x || screen_x > map_x + map_width || 
               screen_y < map_y || screen_y > map_y + map_height {
                continue;
            }
            
            let resource_color = get_resource_color(resource.resource_type);
            draw_circle(screen_x, screen_y, 2.0, resource_color);
        }
    }
}

fn draw_grid(map_x: f32, map_y: f32, map_width: f32, map_height: f32, world_left: f32, world_top: f32, zoom: f32) {
    let grid_size = TILE_SIZE * 10.0; // Draw grid every 10 tiles
    
    // Calculate grid start positions
    let start_x = ((world_left / grid_size).floor() * grid_size - world_left) * zoom;
    let start_y = ((world_top / grid_size).floor() * grid_size - world_top) * zoom;
    
    let grid_color = Color::new(0.2, 0.2, 0.2, 0.5);
    
    // Draw vertical lines
    let mut x = start_x;
    while x < map_width {
        if x >= 0.0 {
            draw_line(
                map_x + x,
                map_y,
                map_x + x,
                map_y + map_height,
                1.0,
                grid_color
            );
        }
        x += grid_size * zoom;
    }
    
    // Draw horizontal lines
    let mut y = start_y;
    while y < map_height {
        if y >= 0.0 {
            draw_line(
                map_x,
                map_y + y,
                map_x + map_width,
                map_y + y,
                1.0,
                grid_color
            );
        }
        y += grid_size * zoom;
    }
}

pub fn is_pilot_window_clicked(game_state: &GameState, mouse_x: f32, mouse_y: f32) -> PilotWindowClick {
    if !game_state.ui_state.pilot_station_open {
        return PilotWindowClick::None;
    }
    
    let screen_width = screen_width();
    let screen_height = screen_height();
    let window_x = (screen_width - PILOT_WINDOW_WIDTH) / 2.0;
    let window_y = (screen_height - PILOT_WINDOW_HEIGHT) / 2.0;
    
    // Check if click is within window
    if mouse_x >= window_x && mouse_x <= window_x + PILOT_WINDOW_WIDTH &&
       mouse_y >= window_y && mouse_y <= window_y + PILOT_WINDOW_HEIGHT {
        
        // Check close button
        let close_x = window_x + PILOT_WINDOW_WIDTH - 30.0;
        if mouse_x >= close_x && mouse_x <= close_x + 30.0 &&
           mouse_y >= window_y && mouse_y <= window_y + 30.0 {
            return PilotWindowClick::Close;
        }
        
        return PilotWindowClick::Inside;
    }
    
    PilotWindowClick::None
}

pub enum PilotWindowClick {
    None,
    Inside,
    Close,
}