use macroquad::prelude::*;
use shared::{types::*, constants::*};
use crate::game_state::*;
use super::utils::*;

pub fn render_effects(game_state: &GameState, cam_x: f32, cam_y: f32) {
    render_weapon_effects(game_state, cam_x, cam_y);
    render_oxygen_tethers(game_state, cam_x, cam_y);
}

fn render_weapon_effects(game_state: &GameState, cam_x: f32, cam_y: f32) {
    for effect in &game_state.weapon_effects {
        if effect.weapon_type == StationType::WeaponLaser {
            if let Some(mech) = game_state.mechs.get(&effect.mech_id) {
                let mech_center = get_mech_center(mech);
                let start_x = cam_x + mech_center.x;
                let start_y = cam_y + mech_center.y;
                let end_x = cam_x + effect.target.x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
                let end_y = cam_y + effect.target.y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
                
                draw_line(
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                    3.0 * effect.timer,
                    Color::new(1.0, 0.0, 0.0, effect.timer)
                );
            }
        }
    }
}

fn render_oxygen_tethers(game_state: &GameState, cam_x: f32, cam_y: f32) {
    for player in game_state.players.values() {
        if let PlayerLocation::OutsideWorld(pos) = player.location {
            if let Some((mech, distance)) = find_nearest_team_mech(game_state, player.team, pos) {
                let mech_center = get_mech_center(mech);
                
                // Calculate tether properties
                let (color, width) = calculate_tether_properties(distance, player.carrying_resource.is_some());
                
                // Draw tether line
                draw_line(
                    cam_x + mech_center.x,
                    cam_y + mech_center.y,
                    cam_x + pos.x,
                    cam_y + pos.y,
                    width,
                    color
                );
            }
        }
    }
}

fn find_nearest_team_mech<'a>(game_state: &'a GameState, team: TeamId, pos: WorldPos) -> Option<(&'a MechState, f32)> {
    game_state.mechs.values()
        .filter(|m| m.team == team)
        .map(|mech| {
            let mech_center = get_mech_center(mech);
            let distance = pos.distance_to(mech_center) / TILE_SIZE;
            (mech, distance)
        })
        .min_by(|(_, dist_a), (_, dist_b)| {
            dist_a.partial_cmp(dist_b).unwrap()
        })
}

fn calculate_tether_properties(distance: f32, carrying_resource: bool) -> (Color, f32) {
    let ratio = (distance / MAX_DISTANCE_FROM_MECH).clamp(0.0, 1.0);
    
    let mut color = Color::new(ratio, 1.0 - ratio, 0.0, 0.7);
    let mut width = 2.0;
    
    // Thicker, more opaque line when carrying resource
    if carrying_resource {
        color.a = 0.9;
        width = 3.0;
    }
    
    // Flash red and thicken line when at limit
    if distance >= MAX_DISTANCE_FROM_MECH {
        color = Color::new(1.0, 0.0, 0.0, 0.9);
        width = 5.0;
    }
    
    (color, width)
}