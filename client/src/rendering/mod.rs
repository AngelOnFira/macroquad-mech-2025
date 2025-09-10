pub mod camera;
mod effects;
pub mod hybrid_tiles;
mod mech_interior;
mod pilot_station;
pub mod primitives;
pub mod spatial_debug;
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

    // Spatial debug rendering
    pub spatial_debug_enabled: bool,
    pub show_coordinate_transforms: bool,
    pub show_mech_bounds: bool,
    pub show_door_positions: bool,
    pub show_coordinate_grid: bool,
    pub show_floor_offsets: bool,
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

            spatial_debug_enabled: false,
            show_coordinate_transforms: false,
            show_mech_bounds: false,
            show_door_positions: false,
            show_coordinate_grid: false,
            show_floor_offsets: false,
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
    pub spatial_debug: spatial_debug::SpatialDebugRenderer,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            spatial_debug: spatial_debug::SpatialDebugRenderer::new(),
        }
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

        // Unified world rendering - always render everything in world space
        {
            #[cfg(feature = "profiling")]
            let _unified_render_span = info_span!("unified_world_render").entered();
            #[cfg(feature = "profiling")]
            scope!("unified_world_render");

            // ALWAYS render world layer (base layer)
            if flags.render_tiles {
                world::render_world_view_with_vision_and_flags(
                    game_state,
                    cam_x,
                    cam_y,
                    vision_system,
                    flags,
                );
            }

            // ALWAYS render all mech interiors in their world positions
            if flags.render_tiles || flags.render_stations || flags.render_players {
                for mech in game_state.mechs.values() {
                    // Render all floors of this mech in world space
                    for floor in 0..shared::MECH_FLOORS as u8 {
                        {
                            #[cfg(feature = "profiling")]
                            scope!("mech_interior_world_space");

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

                            if flags.render_stations {
                                // Skip floor 0 station rendering since render_mech_first_floor() handles it
                                if floor != 0 {
                                    mech_interior::render_stations_on_floor_with_vision(
                                        game_state,
                                        mech.id,
                                        floor,
                                        cam_x,
                                        cam_y,
                                        vision_system,
                                    );
                                }
                            }

                            if flags.render_players {
                                mech_interior::render_players_on_floor_with_vision(
                                    game_state,
                                    mech.id,
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

            // Render effects in world space
            if flags.render_effects {
                #[cfg(feature = "profiling")]
                scope!("effects");
                effects::render_effects(game_state, cam_x, cam_y);
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

        // Render spatial debug overlays (if enabled in debug overlay)
        if flags.spatial_debug_enabled {
            #[cfg(feature = "profiling")]
            scope!("spatial_debug");

            if flags.show_coordinate_grid {
                self.spatial_debug.render_coordinate_grid(cam_x, cam_y);
            }

            if flags.show_mech_bounds {
                self.spatial_debug
                    .render_mech_spatial_bounds(game_state, cam_x, cam_y);
            }

            if flags.show_door_positions {
                self.spatial_debug
                    .render_door_entry_points(game_state, cam_x, cam_y);
            }

            if flags.show_floor_offsets {
                self.spatial_debug
                    .render_floor_offsets(game_state, cam_x, cam_y);
            }

            // Render coordinate mapping if player is inside a mech and coordinate transforms are enabled
            if flags.show_coordinate_transforms {
                if let PlayerLocation::InsideMech {
                    mech_id,
                    pos,
                } = game_state.player_location
                {
                    // Convert MechInteriorPos to WorldPos for the method call
                    let world_pos = pos.tile_pos.to_world_center();
                    self.spatial_debug
                        .render_coordinate_mapping(game_state, mech_id, world_pos, pos.floor, cam_x, cam_y);
                }
            }
        }
    }
}
