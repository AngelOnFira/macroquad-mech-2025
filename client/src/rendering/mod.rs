mod utils;
mod world;
mod effects;
mod mech_interior;
mod ui;
mod pilot_station;

use crate::game_state::*;
use shared::types::*;

pub use pilot_station::{is_pilot_window_clicked, PilotWindowClick};

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

        // Check if we're in a transition
        if let Some(transition) = &game_state.transition {
            self.render_transition(game_state, transition, cam_x, cam_y);
        } else {
            // Normal rendering
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
        }

        // Render UI overlay
        ui::render_ui(game_state);
        
        // Render pilot station window if open
        pilot_station::render_pilot_station_window(game_state);
    }

    fn render_transition(&self, game_state: &GameState, transition: &crate::game_state::TransitionState, cam_x: f32, cam_y: f32) {
        use macroquad::prelude::*;
        use crate::game_state::TransitionType;
        
        // For entering mech: fade from outside to inside
        // For exiting mech: fade from inside to outside
        
        let (first_alpha, second_alpha) = match transition.transition_type {
            TransitionType::EnteringMech => {
                // Fade out the outside world, fade in the mech interior
                (1.0 - transition.progress, transition.progress)
            }
            TransitionType::ExitingMech => {
                // Fade out the mech interior, fade in the outside world
                (1.0 - transition.progress, transition.progress)
            }
        };

        // Render the first view (what we're transitioning from)
        match &transition.from_location {
            PlayerLocation::OutsideWorld(_) => {
                world::render_world_view(game_state, cam_x, cam_y);
                effects::render_effects(game_state, cam_x, cam_y);
            }
            PlayerLocation::InsideMech { mech_id, floor, .. } => {
                if let Some(mech) = game_state.mechs.get(&mech_id) {
                    mech_interior::render_mech_interior(mech, *floor, cam_x, cam_y);
                    mech_interior::render_stations_on_floor(game_state, *mech_id, *floor);
                    mech_interior::render_players_on_floor(game_state, *mech_id, *floor, cam_x, cam_y);
                }
            }
        }

        // Apply fade overlay for the first view
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 1.0 - first_alpha)
        );

        // If we're far enough into the transition, start rendering the second view
        if transition.progress > 0.3 {
            // Render the second view (what we're transitioning to)
            match &transition.to_location {
                PlayerLocation::OutsideWorld(_) => {
                    world::render_world_view(game_state, cam_x, cam_y);
                    effects::render_effects(game_state, cam_x, cam_y);
                }
                PlayerLocation::InsideMech { mech_id, floor, .. } => {
                    if let Some(mech) = game_state.mechs.get(&mech_id) {
                        mech_interior::render_mech_interior(mech, *floor, cam_x, cam_y);
                        mech_interior::render_stations_on_floor(game_state, *mech_id, *floor);
                        mech_interior::render_players_on_floor(game_state, *mech_id, *floor, cam_x, cam_y);
                    }
                }
            }

            // Apply fade overlay for the second view
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, 1.0 - second_alpha)
            );
        }
    }
}