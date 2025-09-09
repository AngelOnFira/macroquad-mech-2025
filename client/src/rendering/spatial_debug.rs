use crate::game_state::GameState;
use macroquad::prelude::*;
use shared::{
    MechInteriorCoordinates, TilePos, WorldPos, FLOOR_HEIGHT_TILES, FLOOR_WIDTH_TILES, MECH_FLOORS,
    TILE_SIZE,
};
use uuid::Uuid;

pub struct SpatialDebugRenderer {
    pub show_coordinate_transforms: bool,
    pub show_mech_bounds: bool,
    pub show_door_positions: bool,
    pub show_coordinate_grid: bool,
    pub show_floor_offsets: bool,
}

impl Default for SpatialDebugRenderer {
    fn default() -> Self {
        Self {
            show_coordinate_transforms: false,
            show_mech_bounds: false,
            show_door_positions: false,
            show_coordinate_grid: false,
            show_floor_offsets: false,
        }
    }
}

impl SpatialDebugRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle all debug visualizations on/off
    pub fn toggle_all(&mut self) {
        let new_state = !self.show_coordinate_transforms;
        self.show_coordinate_transforms = new_state;
        self.show_mech_bounds = new_state;
        self.show_door_positions = new_state;
        self.show_coordinate_grid = new_state;
        self.show_floor_offsets = new_state;
    }

    /// Render coordinate transformation visualization
    pub fn render_coordinate_mapping(
        &self,
        game_state: &GameState,
        mech_id: Uuid,
        player_interior_pos: WorldPos,
        floor: u8,
        cam_x: f32,
        cam_y: f32,
    ) {
        if !self.show_coordinate_transforms {
            return;
        }

        if let Some(mech) = game_state.mechs.get(&mech_id) {
            // Get the player's interior tile position
            let interior_tile = player_interior_pos.to_tile();

            // Transform to world coordinates
            let world_tile =
                MechInteriorCoordinates::interior_to_world(mech.position, floor, interior_tile);
            let world_pos = world_tile.to_world();

            // Draw connection line between interior and world coordinates
            let interior_screen_x = cam_x + player_interior_pos.x;
            let interior_screen_y = cam_y + player_interior_pos.y;
            let world_screen_x = cam_x + world_pos.x;
            let world_screen_y = cam_y + world_pos.y;

            // Draw transformation arrow
            draw_line(
                interior_screen_x,
                interior_screen_y,
                world_screen_x,
                world_screen_y,
                3.0,
                YELLOW,
            );

            // Draw labels
            draw_text(
                &format!(
                    "Interior: {:.1}, {:.1} (Floor {})",
                    player_interior_pos.x / TILE_SIZE,
                    player_interior_pos.y / TILE_SIZE,
                    floor
                ),
                interior_screen_x + 10.0,
                interior_screen_y - 10.0,
                16.0,
                YELLOW,
            );

            draw_text(
                &format!(
                    "World: {:.1}, {:.1}",
                    world_pos.x / TILE_SIZE,
                    world_pos.y / TILE_SIZE
                ),
                world_screen_x + 10.0,
                world_screen_y + 10.0,
                16.0,
                ORANGE,
            );
        }
    }

    /// Render mech spatial bounds for all floors
    pub fn render_mech_spatial_bounds(&self, game_state: &GameState, cam_x: f32, cam_y: f32) {
        if !self.show_mech_bounds {
            return;
        }

        for mech in game_state.mechs.values() {
            self.render_single_mech_bounds(mech.position, cam_x, cam_y);
        }
    }

    fn render_single_mech_bounds(&self, mech_pos: TilePos, cam_x: f32, cam_y: f32) {
        let (min_bounds, max_bounds) = MechInteriorCoordinates::get_mech_world_bounds(mech_pos);

        // Different colors for different floors
        let floor_colors = [
            Color::new(0.2, 0.6, 1.0, 0.3), // Light blue for floor 0
            Color::new(0.2, 1.0, 0.6, 0.3), // Light green for floor 1
            Color::new(1.0, 0.6, 0.2, 0.3), // Light orange for floor 2
        ];

        // Draw bounds for each floor
        for floor in 0..MECH_FLOORS as u8 {
            let color = floor_colors[floor as usize % floor_colors.len()];

            // Calculate floor-specific bounds
            let floor_min =
                MechInteriorCoordinates::interior_to_world(mech_pos, floor, TilePos::new(0, 0));
            let floor_max = MechInteriorCoordinates::interior_to_world(
                mech_pos,
                floor,
                TilePos::new(FLOOR_WIDTH_TILES - 1, FLOOR_HEIGHT_TILES - 1),
            );

            let min_world = floor_min.to_world();
            let max_world = floor_max.to_world();

            // Draw semi-transparent rectangle showing floor bounds in world space
            draw_rectangle(
                cam_x + min_world.x,
                cam_y + min_world.y,
                max_world.x - min_world.x + TILE_SIZE,
                max_world.y - min_world.y + TILE_SIZE,
                color,
            );

            // Draw floor label
            draw_text(
                &format!("Floor {}", floor),
                cam_x + min_world.x + 5.0,
                cam_y + min_world.y + 15.0,
                14.0,
                WHITE,
            );
        }
    }

    /// Render door positions and entry points
    pub fn render_door_entry_points(&self, game_state: &GameState, cam_x: f32, cam_y: f32) {
        if !self.show_door_positions {
            return;
        }

        for mech in game_state.mechs.values() {
            let doors = shared::coordinates::MechDoorPositions::from_mech_position(mech.position);

            // Draw door tiles with special highlighting
            for door_tile in doors.door_tiles() {
                let world_pos = door_tile.to_world();
                let screen_x = cam_x + world_pos.x;
                let screen_y = cam_y + world_pos.y;

                // Draw door outline
                draw_rectangle_lines(screen_x, screen_y, TILE_SIZE, TILE_SIZE, 3.0, GREEN);

                // Draw entry arrow pointing into mech
                let center_x = screen_x + TILE_SIZE / 2.0;
                let center_y = screen_y + TILE_SIZE / 2.0;
                let arrow_size = TILE_SIZE * 0.3;

                // Determine arrow direction based on which door this is
                let mech_center = mech.position.to_world();
                let mech_center_x =
                    mech_center.x + (shared::MECH_SIZE_TILES as f32 * TILE_SIZE) / 2.0;

                if world_pos.x < mech_center_x {
                    // Left door - arrow points right (into mech)
                    self.draw_arrow(center_x, center_y, arrow_size, 0.0, GREEN);
                } else {
                    // Right door - arrow points left (into mech)
                    self.draw_arrow(center_x, center_y, arrow_size, 180.0, GREEN);
                }

                // Draw label
                draw_text("DOOR", screen_x, screen_y - 5.0, 12.0, GREEN);
            }
        }
    }

    /// Draw coordinate grid overlay
    pub fn render_coordinate_grid(&self, cam_x: f32, cam_y: f32) {
        if !self.show_coordinate_grid {
            return;
        }

        let screen_width = screen_width();
        let screen_height = screen_height();

        // Draw world coordinate grid
        let grid_spacing = TILE_SIZE * 5.0; // Every 5 tiles
        let line_color = Color::new(1.0, 1.0, 1.0, 0.2);

        // Vertical lines
        let start_x = ((cam_x / grid_spacing).floor() * grid_spacing).max(-cam_x);
        let mut x = start_x;
        while x - cam_x < screen_width {
            let screen_x = x + cam_x;
            if screen_x >= 0.0 && screen_x <= screen_width {
                draw_line(screen_x, 0.0, screen_x, screen_height, 1.0, line_color);

                // Grid coordinates label
                let grid_coord = (x / TILE_SIZE) as i32;
                draw_text(
                    &format!("{}", grid_coord),
                    screen_x + 2.0,
                    15.0,
                    12.0,
                    line_color,
                );
            }
            x += grid_spacing;
        }

        // Horizontal lines
        let start_y = ((cam_y / grid_spacing).floor() * grid_spacing).max(-cam_y);
        let mut y = start_y;
        while y - cam_y < screen_height {
            let screen_y = y + cam_y;
            if screen_y >= 0.0 && screen_y <= screen_height {
                draw_line(0.0, screen_y, screen_width, screen_y, 1.0, line_color);

                // Grid coordinates label
                let grid_coord = (y / TILE_SIZE) as i32;
                draw_text(
                    &format!("{}", grid_coord),
                    5.0,
                    screen_y - 2.0,
                    12.0,
                    line_color,
                );
            }
            y += grid_spacing;
        }
    }

    /// Render floor Y-offset visualization
    pub fn render_floor_offsets(&self, game_state: &GameState, cam_x: f32, cam_y: f32) {
        if !self.show_floor_offsets {
            return;
        }

        // Show virtual Y-offset mapping
        let legend_x = 10.0;
        let mut legend_y = 100.0;

        draw_text("Floor Y-Offset Mapping:", legend_x, legend_y, 16.0, WHITE);
        legend_y += 20.0;

        for floor in 0..MECH_FLOORS as u8 {
            let virtual_y_offset = floor as i32 * (shared::FLOOR_HEIGHT_TILES + 1);
            let color = match floor {
                0 => BLUE,
                1 => GREEN,
                2 => ORANGE,
                _ => WHITE,
            };

            draw_text(
                &format!("Floor {}: Y offset = {}", floor, virtual_y_offset),
                legend_x,
                legend_y,
                14.0,
                color,
            );
            legend_y += 18.0;
        }

        // Draw actual offset visualization for each mech
        for mech in game_state.mechs.values() {
            for floor in 0..MECH_FLOORS as u8 {
                let base_pos = MechInteriorCoordinates::interior_to_world(
                    mech.position,
                    floor,
                    TilePos::new(0, 0),
                );
                let world_pos = base_pos.to_world();

                let screen_x = cam_x + world_pos.x;
                let screen_y = cam_y + world_pos.y;

                // Draw offset indicator
                let color = match floor {
                    0 => BLUE,
                    1 => GREEN,
                    2 => ORANGE,
                    _ => WHITE,
                };

                draw_circle(screen_x, screen_y, 3.0, color);
                draw_text(
                    &format!("F{}", floor),
                    screen_x + 5.0,
                    screen_y,
                    12.0,
                    color,
                );
            }
        }
    }

    /// Helper function to draw arrows
    fn draw_arrow(
        &self,
        center_x: f32,
        center_y: f32,
        size: f32,
        angle_degrees: f32,
        color: Color,
    ) {
        let angle_rad = angle_degrees.to_radians();

        // Arrow tip
        let tip_x = center_x + (angle_rad.cos() * size);
        let tip_y = center_y + (angle_rad.sin() * size);

        // Arrow base points
        let base_angle1 = angle_rad + 2.5;
        let base_angle2 = angle_rad - 2.5;

        let base1_x = center_x + (base_angle1.cos() * size * 0.6);
        let base1_y = center_y + (base_angle1.sin() * size * 0.6);

        let base2_x = center_x + (base_angle2.cos() * size * 0.6);
        let base2_y = center_y + (base_angle2.sin() * size * 0.6);

        // Draw arrow triangle
        draw_triangle(
            Vec2::new(tip_x, tip_y),
            Vec2::new(base1_x, base1_y),
            Vec2::new(base2_x, base2_y),
            color,
        );
    }

    /// Render debug info panel
    pub fn render_debug_panel(
        &self,
        game_state: &GameState,
        current_player_location: Option<&shared::types::PlayerLocation>,
        spatial_test_status: Option<&str>,
    ) {
        let panel_x = screen_width() - 300.0;
        let panel_y = 10.0;
        let panel_width = 290.0;
        let panel_height = 200.0;

        // Semi-transparent background
        draw_rectangle(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        draw_rectangle_lines(panel_x, panel_y, panel_width, panel_height, 2.0, WHITE);

        let mut y = panel_y + 20.0;
        draw_text("Spatial Debug Info", panel_x + 10.0, y, 16.0, WHITE);
        y += 25.0;

        // Current player location info
        if let Some(location) = current_player_location {
            match location {
                shared::types::PlayerLocation::OutsideWorld(pos) => {
                    draw_text(
                        &format!("Location: Outside World"),
                        panel_x + 10.0,
                        y,
                        14.0,
                        YELLOW,
                    );
                    y += 18.0;
                    draw_text(
                        &format!(
                            "World: ({:.1}, {:.1})",
                            pos.x / TILE_SIZE,
                            pos.y / TILE_SIZE
                        ),
                        panel_x + 10.0,
                        y,
                        12.0,
                        WHITE,
                    );
                    y += 18.0;
                }
                shared::types::PlayerLocation::InsideMech {
                    mech_id,
                    pos,
                } => {
                    let floor = pos.floor();
                    draw_text(
                        &format!("Location: Inside Mech"),
                        panel_x + 10.0,
                        y,
                        14.0,
                        GREEN,
                    );
                    y += 18.0;
                    draw_text(
                        &format!("Mech ID: {:.8}", mech_id.to_string()),
                        panel_x + 10.0,
                        y,
                        12.0,
                        WHITE,
                    );
                    y += 18.0;
                    draw_text(&format!("Floor: {}", floor), panel_x + 10.0, y, 12.0, WHITE);
                    y += 18.0;
                    draw_text(
                        &format!(
                            "Interior: ({:.1}, {:.1})",
                            pos.tile_pos().x as f32,
                            pos.tile_pos().y as f32
                        ),
                        panel_x + 10.0,
                        y,
                        12.0,
                        WHITE,
                    );
                    y += 18.0;

                    // Calculate and show equivalent world position
                    if let Some(mech) = game_state.mechs.get(mech_id) {
                        let interior_tile = pos.tile_pos();
                        let world_tile = MechInteriorCoordinates::interior_to_world(
                            mech.position,
                            floor,
                            interior_tile,
                        );
                        let world_pos = world_tile.to_world();
                        draw_text(
                            &format!(
                                "World Equiv: ({:.1}, {:.1})",
                                world_pos.x / TILE_SIZE,
                                world_pos.y / TILE_SIZE
                            ),
                            panel_x + 10.0,
                            y,
                            12.0,
                            ORANGE,
                        );
                        y += 18.0;
                    }
                }
            }
        }

        // Test status
        if let Some(test_status) = spatial_test_status {
            y += 10.0;
            draw_text("Spatial Test:", panel_x + 10.0, y, 14.0, YELLOW);
            y += 18.0;
            draw_text(test_status, panel_x + 10.0, y, 12.0, GREEN);
            y += 18.0;
        }

        // Controls help
        y += 10.0;
        draw_text("Controls:", panel_x + 10.0, y, 14.0, YELLOW);
        y += 18.0;
        draw_text("F1 - Toggle spatial debug", panel_x + 10.0, y, 12.0, WHITE);
        y += 18.0;
        draw_text("F2 - Start/Stop coord test", panel_x + 10.0, y, 12.0, WHITE);
        y += 18.0;
        draw_text(
            "F3 - Start/Stop movement test",
            panel_x + 10.0,
            y,
            12.0,
            WHITE,
        );
        y += 18.0;
        draw_text("F4 - Show test report", panel_x + 10.0, y, 12.0, WHITE);
    }
}
