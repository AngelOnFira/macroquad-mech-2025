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

#[derive(Clone)]
pub struct RenderFlags {
    pub render_mechs: bool,
    pub render_players: bool,
    pub render_resources: bool,
    pub render_projectiles: bool,
    pub render_effects: bool,
    pub render_ui: bool,
    pub render_fog: bool,
    pub render_tiles: bool,
    pub render_stations: bool,
}

impl Default for RenderFlags {
    fn default() -> Self {
        Self {
            render_mechs: true,
            render_players: true,
            render_resources: true,
            render_projectiles: true,
            render_effects: true,
            render_ui: true,
            render_fog: true,
            render_tiles: true,
            render_stations: true,
        }
    }
}

#[cfg(feature = "profiling")]
use profiling::scope;

#[cfg(feature = "profiling")]
use tracing::info_span;

pub use pilot_station::{is_pilot_window_clicked, PilotWindowClick};

pub struct Renderer {
    // Could store textures and other rendering resources here
}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, game_state: &GameState) {
        self.render_with_flags(game_state, &RenderFlags::default());
    }
    
    pub fn render_with_flags(&self, game_state: &GameState, flags: &RenderFlags) {
        #[cfg(feature = "profiling")]
        let _renderer_span = info_span!("renderer").entered();
        #[cfg(feature = "profiling")]
        scope!("renderer");

        // Apply camera transform
        let cam_x = -game_state.camera_offset.0;
        let cam_y = -game_state.camera_offset.1;
        
        // Only use vision system if fog of war is enabled
        let vision_system = if flags.render_fog {
            Some(&game_state.vision_system)
        } else {
            None
        };

        // Check if we're in a transition
        if let Some(transition) = &game_state.transition {
            #[cfg(feature = "profiling")]
            let _transition_span = info_span!("transition").entered();
            #[cfg(feature = "profiling")]
            scope!("transition");

            self.render_transition(game_state, transition, cam_x, cam_y, vision_system);
        } else {
            // Normal rendering
            match game_state.player_location {
                PlayerLocation::OutsideWorld(_) => {
                    {
                        #[cfg(feature = "profiling")]
                        let _world_view_span = info_span!("world_view").entered();
                        #[cfg(feature = "profiling")]
                        scope!("world_view");

                        world::render_world_view_with_vision_and_flags(
                            game_state,
                            cam_x,
                            cam_y,
                            vision_system,
                            flags,
                        );
                    }
                    {
                        #[cfg(feature = "profiling")]
                        let _effects_span = info_span!("effects").entered();
                        #[cfg(feature = "profiling")]
                        scope!("effects");

                        if flags.render_effects {
                            effects::render_effects(game_state, cam_x, cam_y);
                        }
                    }
                }
                PlayerLocation::InsideMech { mech_id, floor, .. } => {
                    #[cfg(feature = "profiling")]
                    scope!("mech_interior");

                    if let Some(mech) = game_state.mechs.get(&mech_id) {
                        {
                            #[cfg(feature = "profiling")]
                            scope!("mech_tiles");

                            if flags.render_tiles {
                                mech_interior::render_mech_interior_with_vision(
                                    game_state,
                                    mech,
                                    floor,
                                    cam_x,
                                    cam_y,
                                    vision_system,
                                );
                            }
                        }
                        {
                            #[cfg(feature = "profiling")]
                            scope!("mech_stations");

                            if flags.render_stations {
                                mech_interior::render_stations_on_floor_with_vision(
                                    game_state,
                                    mech_id,
                                    floor,
                                    cam_x,
                                    cam_y,
                                    vision_system,
                                );
                            }
                        }
                        {
                            #[cfg(feature = "profiling")]
                            scope!("mech_players");

                            if flags.render_players {
                                mech_interior::render_players_on_floor_with_vision(
                                    game_state,
                                    mech_id,
                                    floor,
                                    cam_x,
                                    cam_y,
                                    vision_system,
                                );
                            }
                        }
                    }
                }
            }
        }

        // Render UI overlay
        if flags.render_ui {
            #[cfg(feature = "profiling")]
            scope!("ui");

            ui::render_ui(game_state);
        }

        // Render pilot station window if open
        if flags.render_ui {
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
        vision_system: Option<&crate::vision::ClientVisionSystem>,
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
                    vision_system,
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
                        vision_system,
                    );
                    mech_interior::render_stations_on_floor_with_vision(
                        game_state,
                        *mech_id,
                        *floor,
                        cam_x,
                        cam_y,
                        vision_system,
                    );
                    mech_interior::render_players_on_floor_with_vision(
                        game_state,
                        *mech_id,
                        *floor,
                        cam_x,
                        cam_y,
                        vision_system,
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
                        vision_system,
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
                            vision_system,
                        );
                        mech_interior::render_stations_on_floor_with_vision(
                            game_state,
                            *mech_id,
                            *floor,
                            cam_x,
                            cam_y,
                            vision_system,
                        );
                        mech_interior::render_players_on_floor_with_vision(
                            game_state,
                            *mech_id,
                            *floor,
                            cam_x,
                            cam_y,
                            vision_system,
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
