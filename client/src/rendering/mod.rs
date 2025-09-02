pub mod camera;
mod effects;
pub mod hybrid_tiles;
mod mech_interior;
mod pilot_station;
pub mod primitives;
mod ui;
mod utils;
mod world;

use crate::game_state::*;
use shared::types::*;

#[cfg(feature = "profiling")]
use profiling::scope;


pub use pilot_station::{is_pilot_window_clicked, PilotWindowClick};

pub struct Renderer {
    // Could store textures and other rendering resources here
}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, game_state: &GameState) {
        #[cfg(feature = "profiling")]
        scope!("renderer");
        
        // Apply camera transform
        let cam_x = -game_state.camera_offset.0;
        let cam_y = -game_state.camera_offset.1;

        // Check if we're in a transition
        if let Some(transition) = &game_state.transition {
            #[cfg(feature = "profiling")]
            scope!("transition");
            
            self.render_transition(game_state, transition, cam_x, cam_y);
        } else {
            // Normal rendering
            match game_state.player_location {
                PlayerLocation::OutsideWorld(_) => {
                    {
                        #[cfg(feature = "profiling")]
                        scope!("world_view");
                        
                        world::render_world_view_with_vision(
                            game_state,
                            cam_x,
                            cam_y,
                            Some(&game_state.vision_system),
                        );
                    }
                    {
                        #[cfg(feature = "profiling")]
                        scope!("effects");
                        
                        effects::render_effects(game_state, cam_x, cam_y);
                    }
                }
                PlayerLocation::InsideMech { mech_id, floor, .. } => {
                    #[cfg(feature = "profiling")]
                    scope!("mech_interior");
                    
                    if let Some(mech) = game_state.mechs.get(&mech_id) {
                        {
                            #[cfg(feature = "profiling")]
                            scope!("mech_tiles");
                            
                            mech_interior::render_mech_interior_with_vision(
                                game_state,
                                mech,
                                floor,
                                cam_x,
                                cam_y,
                                Some(&game_state.vision_system),
                            );
                        }
                        {
                            #[cfg(feature = "profiling")]
                            scope!("mech_stations");
                            
                            mech_interior::render_stations_on_floor_with_vision(
                                game_state,
                                mech_id,
                                floor,
                                cam_x,
                                cam_y,
                                Some(&game_state.vision_system),
                            );
                        }
                        {
                            #[cfg(feature = "profiling")]
                            scope!("mech_players");
                            
                            mech_interior::render_players_on_floor_with_vision(
                                game_state,
                                mech_id,
                                floor,
                                cam_x,
                                cam_y,
                                Some(&game_state.vision_system),
                            );
                        }
                    }
                }
            }
        }

        // Render UI overlay
        {
            #[cfg(feature = "profiling")]
            scope!("ui");
            
            ui::render_ui(game_state);
        }

        // Render pilot station window if open
        {
            #[cfg(feature = "profiling")]
            scope!("pilot_station");
            
            pilot_station::render_pilot_station_window(game_state);
        }
    }

    fn render_transition(
        &self,
        game_state: &GameState,
        transition: &crate::game_state::TransitionState,
        cam_x: f32,
        cam_y: f32,
    ) {
        #[cfg(feature = "profiling")]
        scope!("transition");
        
        use crate::game_state::TransitionType;
        use macroquad::prelude::*;

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
                world::render_world_view_with_vision(
                    game_state,
                    cam_x,
                    cam_y,
                    Some(&game_state.vision_system),
                );
                effects::render_effects(game_state, cam_x, cam_y);
            }
            PlayerLocation::InsideMech { mech_id, floor, .. } => {
                if let Some(mech) = game_state.mechs.get(&mech_id) {
                    mech_interior::render_mech_interior_with_vision(
                        game_state,
                        mech,
                        *floor,
                        cam_x,
                        cam_y,
                        Some(&game_state.vision_system),
                    );
                    mech_interior::render_stations_on_floor_with_vision(
                        game_state,
                        *mech_id,
                        *floor,
                        cam_x,
                        cam_y,
                        Some(&game_state.vision_system),
                    );
                    mech_interior::render_players_on_floor_with_vision(
                        game_state,
                        *mech_id,
                        *floor,
                        cam_x,
                        cam_y,
                        Some(&game_state.vision_system),
                    );
                }
            }
        }

        // Apply fade overlay for the first view
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 1.0 - first_alpha),
        );

        // If we're far enough into the transition, start rendering the second view
        if transition.progress > 0.3 {
            // Render the second view (what we're transitioning to)
            match &transition.to_location {
                PlayerLocation::OutsideWorld(_) => {
                    world::render_world_view_with_vision(
                        game_state,
                        cam_x,
                        cam_y,
                        Some(&game_state.vision_system),
                    );
                    effects::render_effects(game_state, cam_x, cam_y);
                }
                PlayerLocation::InsideMech { mech_id, floor, .. } => {
                    if let Some(mech) = game_state.mechs.get(&mech_id) {
                        mech_interior::render_mech_interior_with_vision(
                            game_state,
                            mech,
                            *floor,
                            cam_x,
                            cam_y,
                            Some(&game_state.vision_system),
                        );
                        mech_interior::render_stations_on_floor_with_vision(
                            game_state,
                            *mech_id,
                            *floor,
                            cam_x,
                            cam_y,
                            Some(&game_state.vision_system),
                        );
                        mech_interior::render_players_on_floor_with_vision(
                            game_state,
                            *mech_id,
                            *floor,
                            cam_x,
                            cam_y,
                            Some(&game_state.vision_system),
                        );
                    }
                }
            }

            // Apply fade overlay for the second view
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, 1.0 - second_alpha),
            );
        }
    }
}
