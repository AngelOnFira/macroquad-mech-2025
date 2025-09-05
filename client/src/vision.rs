use crate::game_state::{GameState, MechState};
use shared::{
    constants::*, coordinates::TileRange, tile_entity::TileVisual, MechDoorPositions,
    MechVisionUtils, PlayerLocation, TilePos, WorldPos, TILE_SIZE,
};
use std::collections::{HashMap, HashSet};

/// Client-side fog of war vision system
pub struct ClientVisionSystem {
    /// Tiles that are currently visible
    pub visible_tiles: HashSet<TilePos>,

    /// Visibility strength for each tile (0.0 = invisible, 1.0 = fully visible)
    pub visibility_mask: HashMap<TilePos, f32>,

    /// Mech interior tiles that are visible from outside
    /// Key: (mech_id, floor, interior_x, interior_y), Value: visibility strength
    pub visible_interior_tiles: HashMap<(uuid::Uuid, u8, i32, i32), f32>,

    /// Last position where visibility was calculated
    pub last_update_pos: WorldPos,

    /// Vision range in tiles
    pub vision_range: TileRange,

    /// Minimum movement required before recalculating visibility
    pub update_threshold: f32,

    /// Frame counter for skipping vision updates
    frame_counter: u32,
}

impl ClientVisionSystem {
    pub fn new() -> Self {
        Self {
            visible_tiles: HashSet::new(),
            visibility_mask: HashMap::new(),
            visible_interior_tiles: HashMap::new(),
            last_update_pos: WorldPos::new(-999.0, -999.0), // Force initial update
            vision_range: TileRange::new(8),                // 8 tiles of vision range
            update_threshold: 16.0, // Half tile movement (increased sensitivity)
            frame_counter: 0,
        }
    }

    /// Update visibility based on player position and location
    pub fn update_visibility(game_state: &mut GameState) {
        // Increment frame counter and skip update if not time yet
        game_state.vision_system.frame_counter += 1;
        if game_state.vision_system.frame_counter % 3 != 0 {
            // Update every 3 frames (20fps)
            return;
        }

        let (player_pos, _player_location) = match Self::get_player_info(game_state) {
            Some(info) => info,
            None => return, // Player not found or not positioned
        };

        // Check if we need to update (player moved significantly)
        if game_state
            .vision_system
            .last_update_pos
            .distance_to(player_pos)
            < game_state.vision_system.update_threshold
        {
            return;
        }

        // Clear previous visibility data
        game_state.vision_system.visible_tiles.clear();
        game_state.vision_system.visibility_mask.clear();
        game_state.vision_system.visible_interior_tiles.clear();

        // Calculate new visibility
        Self::calculate_visibility(game_state, player_pos);

        // Update last position
        game_state.vision_system.last_update_pos = player_pos;
    }

    /// Force a visibility update regardless of movement
    pub fn force_update(game_state: &mut GameState) {
        game_state.vision_system.last_update_pos = WorldPos::new(-999.0, -999.0);
        Self::update_visibility(game_state);
    }

    /// Get the visibility strength for a world tile (0.0 to 1.0)
    pub fn get_visibility(&self, tile_pos: TilePos) -> f32 {
        self.visibility_mask.get(&tile_pos).copied().unwrap_or(0.0)
    }

    /// Check if a tile is visible
    pub fn is_visible(&self, tile_pos: TilePos) -> bool {
        self.visible_tiles.contains(&tile_pos)
    }

    /// Get visibility for a mech interior tile
    pub fn get_interior_visibility(
        &self,
        mech_id: uuid::Uuid,
        floor: u8,
        interior_pos: TilePos,
    ) -> f32 {
        let key = (mech_id, floor, interior_pos.x, interior_pos.y);
        self.visible_interior_tiles
            .get(&key)
            .copied()
            .unwrap_or(0.0)
    }

    /// Check if a mech interior tile is visible
    pub fn is_interior_visible(
        &self,
        mech_id: uuid::Uuid,
        floor: u8,
        interior_pos: TilePos,
    ) -> bool {
        self.get_interior_visibility(mech_id, floor, interior_pos) > 0.1
    }

    /// Get all visible interior tiles for a specific mech
    pub fn get_visible_interior_for_mech(&self, mech_id: uuid::Uuid) -> Vec<(u8, TilePos, f32)> {
        self.visible_interior_tiles
            .iter()
            .filter_map(|((m_id, floor, x, y), &visibility)| {
                if *m_id == mech_id {
                    Some((*floor, TilePos::new(*x, *y), visibility))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Internal: Get player position and location
    fn get_player_info(game_state: &GameState) -> Option<(WorldPos, PlayerLocation)> {
        let player_id = game_state.player_id?;
        let player = game_state.players.get(&player_id)?;

        let world_pos = match player.location {
            PlayerLocation::OutsideWorld(pos) => pos,
            PlayerLocation::InsideMech { pos, .. } => pos,
        };

        Some((world_pos, player.location))
    }

    /// Internal: Main visibility calculation
    fn calculate_visibility(game_state: &mut GameState, viewer_pos: WorldPos) {
        // Cast rays in multiple directions for 360-degree vision
        let num_rays = 72; // Every 5 degrees (reduced from 360 for performance)
        let angle_step = std::f32::consts::PI * 2.0 / num_rays as f32;

        for i in 0..num_rays {
            let angle = i as f32 * angle_step;
            Self::cast_vision_ray(game_state, viewer_pos, angle);
        }

        // Calculate visibility for mech interiors
        Self::calculate_mech_interior_visibility(game_state, viewer_pos);
    }

    /// Internal: Cast a single vision ray
    fn cast_vision_ray(game_state: &mut GameState, start_pos: WorldPos, angle: f32) {
        let dx = angle.cos();
        let dy = angle.sin();
        let step_size = TILE_SIZE * 0.5; // Half-tile steps (better performance)

        let mut current_pos = start_pos;
        let mut distance = 0.0;
        let mut vision_blocked = false;

        while distance < game_state.vision_system.vision_range.to_world_distance()
            && !vision_blocked
        {
            let tile_pos = current_pos.to_tile();

            // Add this tile to visible set
            game_state.vision_system.visible_tiles.insert(tile_pos);

            // Calculate visibility strength based on distance
            let base_visibility = (1.0
                - (distance / game_state.vision_system.vision_range.to_world_distance()))
            .max(0.0);

            // Check what's at this tile position
            let mut tile_visibility = base_visibility;
            let mut blocks_further_vision = false;

            // Check world tiles first
            if let Some((_, tile_visual)) = game_state
                .visible_tiles
                .iter()
                .find(|(pos, _)| **pos == tile_pos)
            {
                match tile_visual {
                    TileVisual::Wall { .. } => {
                        blocks_further_vision = true;
                        tile_visibility *= 0.1; // Can barely see the wall itself
                    }
                    TileVisual::Window { .. } => {
                        tile_visibility *= 0.8; // Slight reduction through glass
                    }
                    _ => {
                        // Most tiles don't affect visibility
                    }
                }
            }

            // Check for mechs at this position
            if let Some(mech) = Self::find_mech_at_position(tile_pos, game_state) {
                // Mech blocks vision unless we're looking through a door/window
                let doors = MechDoorPositions::from_mech_position(mech.position);

                if doors.is_door_tile(tile_pos) {
                    // Door - reduces visibility but doesn't completely block
                    tile_visibility *= 0.7;
                } else {
                    // Solid mech wall - blocks vision significantly
                    blocks_further_vision = true;
                    tile_visibility *= 0.2;
                }
            }

            // Update visibility mask
            let current_visibility = game_state
                .vision_system
                .visibility_mask
                .get(&tile_pos)
                .copied()
                .unwrap_or(0.0);
            game_state
                .vision_system
                .visibility_mask
                .insert(tile_pos, current_visibility.max(tile_visibility));

            // Stop casting if vision is blocked
            if blocks_further_vision {
                vision_blocked = true;
            }

            // Advance ray
            current_pos.x += dx * step_size;
            current_pos.y += dy * step_size;
            distance += step_size;
        }
    }

    /// Internal: Calculate visibility for mech interior tiles
    fn calculate_mech_interior_visibility(game_state: &mut GameState, viewer_pos: WorldPos) {
        // For each mech, check if we can see any interior tiles
        for mech in game_state.mechs.values() {
            let visible_interior_tiles = MechVisionUtils::get_potentially_visible_interior_tiles(
                viewer_pos,
                mech.position,
                game_state.vision_system.vision_range.to_world_distance(),
            );

            for (floor, interior_pos, visibility) in visible_interior_tiles {
                let key = (mech.id, floor, interior_pos.x, interior_pos.y);
                game_state
                    .vision_system
                    .visible_interior_tiles
                    .insert(key, visibility);
            }
        }
    }

    /// Internal: Find mech at a specific world position
    fn find_mech_at_position<'a>(
        tile_pos: TilePos,
        game_state: &'a GameState,
    ) -> Option<&'a MechState> {
        game_state.mechs.values().find(|mech| {
            // Check if this tile position is within the mech's footprint
            tile_pos.x >= mech.position.x
                && tile_pos.x < mech.position.x + MECH_SIZE_TILES
                && tile_pos.y >= mech.position.y
                && tile_pos.y < mech.position.y + MECH_SIZE_TILES
        })
    }
}

/// Helper struct for managing fog of war rendering effects
pub struct FogOfWarRenderer;

impl FogOfWarRenderer {
    /// Apply fog of war effect to a color based on visibility
    pub fn apply_fog_to_color(
        base_color: macroquad::color::Color,
        visibility: f32,
    ) -> macroquad::color::Color {
        let fog_strength = 1.0 - visibility.clamp(0.0, 1.0);

        // Blend towards dark gray
        let fog_color = macroquad::color::Color::new(0.1, 0.1, 0.1, 1.0);

        macroquad::color::Color::new(
            base_color.r * (1.0 - fog_strength) + fog_color.r * fog_strength,
            base_color.g * (1.0 - fog_strength) + fog_color.g * fog_strength,
            base_color.b * (1.0 - fog_strength) + fog_color.b * fog_strength,
            base_color.a, // Preserve alpha
        )
    }

    /// Get fog overlay color for completely invisible areas
    pub fn get_fog_overlay_color() -> macroquad::color::Color {
        macroquad::color::Color::new(0.0, 0.0, 0.0, 0.9) // Dark overlay
    }

    /// Calculate smooth fog transition based on distance from visible edge
    pub fn calculate_edge_fade(
        tile_pos: TilePos,
        vision_system: &ClientVisionSystem,
        fade_distance: i32,
    ) -> f32 {
        // If tile is visible, no fade needed
        if vision_system.is_visible(tile_pos) {
            return 1.0;
        }

        // return 1.0;

        // Find distance to nearest visible tile
        let mut min_distance = fade_distance + 1;

        for dx in -fade_distance..=fade_distance {
            for dy in -fade_distance..=fade_distance {
                let check_pos = TilePos::new(tile_pos.x + dx, tile_pos.y + dy);
                if vision_system.is_visible(check_pos) {
                    let distance = dx.abs().max(dy.abs()); // Chebyshev distance
                    min_distance = min_distance.min(distance);
                }
            }
        }

        if min_distance <= fade_distance {
            // Smooth fade from visible edge
            1.0 - (min_distance as f32 / fade_distance as f32)
        } else {
            0.0 // Completely fogged
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::TeamId;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_game_state() -> GameState {
        // Create a minimal game state for testing
        GameState {
            player_id: Some(Uuid::new_v4()),
            player_location: PlayerLocation::OutsideWorld(WorldPos::new(100.0, 100.0)),
            player_team: Some(TeamId::Red),
            players: HashMap::new(),
            mechs: HashMap::new(),
            stations: HashMap::new(),
            resources: Vec::new(),
            projectiles: Vec::new(),
            weapon_effects: Vec::new(),
            camera_offset: (0.0, 0.0),
            ui_state: crate::game_state::UIState {
                pilot_station_open: false,
                pilot_station_id: None,
                operating_mech_id: None,
            },
            visible_tiles: HashMap::new(),
            vision_system: ClientVisionSystem::new(),
        }
    }

    #[test]
    fn test_vision_system_creation() {
        let vision = ClientVisionSystem::new();
        assert!(vision.visible_tiles.is_empty());
        assert!(vision.visibility_mask.is_empty());
        assert_eq!(vision.vision_range.tiles(), 8);
    }

    #[test]
    fn test_visibility_functions() {
        let mut vision = ClientVisionSystem::new();
        let tile_pos = TilePos::new(5, 5);

        // Initially not visible
        assert!(!vision.is_visible(tile_pos));
        assert_eq!(vision.get_visibility(tile_pos), 0.0);

        // Make visible
        vision.visible_tiles.insert(tile_pos);
        vision.visibility_mask.insert(tile_pos, 0.7);

        assert!(vision.is_visible(tile_pos));
        assert_eq!(vision.get_visibility(tile_pos), 0.7);
    }

    #[test]
    fn test_fog_of_war_renderer() {
        use macroquad::color::Color;

        let base_color = Color::new(1.0, 0.0, 0.0, 1.0); // Red

        // Full visibility - no change
        let full_vis = FogOfWarRenderer::apply_fog_to_color(base_color, 1.0);
        assert_eq!(full_vis, base_color);

        // No visibility - heavily fogged
        let no_vis = FogOfWarRenderer::apply_fog_to_color(base_color, 0.0);
        assert!(no_vis.r < 0.5); // Should be much darker

        // Partial visibility
        let partial_vis = FogOfWarRenderer::apply_fog_to_color(base_color, 0.5);
        assert!(partial_vis.r > no_vis.r && partial_vis.r < full_vis.r);
    }
}
