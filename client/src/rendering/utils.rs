use macroquad::prelude::*;
use shared::types::*;
use shared::constants::*;
use crate::game_state::MechState;

/// Get the color for a resource type
pub fn get_resource_color(resource_type: ResourceType) -> Color {
    match resource_type {
        ResourceType::ScrapMetal => DARKGRAY,
        ResourceType::ComputerComponents => GREEN,
        ResourceType::Wiring => YELLOW,
        ResourceType::Batteries => ORANGE,
    }
}

/// Get the color for a team
pub fn get_team_color(team: TeamId) -> Color {
    match team {
        TeamId::Red => Color::new(0.8, 0.2, 0.2, 1.0),
        TeamId::Blue => Color::new(0.2, 0.2, 0.8, 1.0),
    }
}

/// Get the color for a player based on their team
pub fn get_player_color(team: TeamId) -> Color {
    match team {
        TeamId::Red => Color::new(1.0, 0.3, 0.3, 1.0),
        TeamId::Blue => Color::new(0.3, 0.3, 1.0, 1.0),
    }
}

/// Get the color for a station type
pub fn get_station_color(station_type: StationType) -> Color {
    match station_type {
        StationType::WeaponLaser => RED,
        StationType::WeaponProjectile => ORANGE,
        StationType::Engine => BLUE,
        StationType::Shield => SKYBLUE,
        StationType::Repair => GREEN,
        StationType::Electrical => YELLOW,
        StationType::Upgrade => PURPLE,
    }
}

/// Calculate the center position of a mech in world coordinates
pub fn get_mech_center(mech: &MechState) -> WorldPos {
    WorldPos::new(
        (mech.position.x as f32 + MECH_SIZE_TILES as f32 / 2.0) * TILE_SIZE,
        (mech.position.y as f32 + MECH_SIZE_TILES as f32 / 2.0) * TILE_SIZE
    )
}

/// Draw a dashed rectangle outline
pub fn draw_dashed_rectangle_lines(x: f32, y: f32, width: f32, height: f32, dash_size: f32, gap_size: f32, thickness: f32, color: Color) {
    let num_dashes_x = (width / (dash_size + gap_size)) as i32;
    let num_dashes_y = (height / (dash_size + gap_size)) as i32;
    
    // Top and bottom edges
    for i in 0..num_dashes_x {
        let dash_x = x + i as f32 * (dash_size + gap_size);
        draw_rectangle(dash_x, y, dash_size, thickness, color);
        draw_rectangle(dash_x, y + height - thickness, dash_size, thickness, color);
    }
    
    // Left and right edges
    for i in 0..num_dashes_y {
        let dash_y = y + i as f32 * (dash_size + gap_size);
        draw_rectangle(x, dash_y, thickness, dash_size, color);
        draw_rectangle(x + width - thickness, dash_y, thickness, dash_size, color);
    }
}