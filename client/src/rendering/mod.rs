mod utils;
mod world;
mod effects;
mod mech_interior;
mod ui;

use crate::game_state::*;
use shared::types::*;

pub struct Renderer {
    // Could store textures and other rendering resources here
}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, game_state: &GameState) {
        // Apply camera transform
        let cam_x = -game_state.camera_offset.0;
        let cam_y = -game_state.camera_offset.1;

        match game_state.player_location {
            PlayerLocation::OutsideWorld(_) => {
                world::render_world_view(game_state, cam_x, cam_y);
                effects::render_effects(game_state, cam_x, cam_y);
            }
            PlayerLocation::InsideMech { mech_id, floor, .. } => {
                if let Some(mech) = game_state.mechs.get(&mech_id) {
                    mech_interior::render_mech_interior(mech, floor, cam_x, cam_y);
                    mech_interior::render_stations_on_floor(game_state, mech_id, floor);
                    mech_interior::render_players_on_floor(game_state, mech_id, floor, cam_x, cam_y);
                }
            }
        }

        // Render UI overlay
        ui::render_ui(game_state);
    }
}