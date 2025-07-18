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
        StationType::Pilot => Color::new(0.5, 0.8, 0.5, 1.0), // Light green
    }
}

/// Calculate the center position of a mech in world coordinates
pub fn get_mech_center(mech: &MechState) -> WorldPos {
    WorldPos::new(
        (mech.position.x as f32 + MECH_SIZE_TILES as f32 / 2.0) * TILE_SIZE,
        (mech.position.y as f32 + MECH_SIZE_TILES as f32 / 2.0) * TILE_SIZE
    )
}

