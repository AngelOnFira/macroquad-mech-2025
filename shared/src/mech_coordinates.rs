use crate::coordinates::{MechDoorPositions, WorldPos};
use crate::types::Direction;
use crate::{TilePos, FLOOR_HEIGHT_TILES, FLOOR_WIDTH_TILES, MECH_FLOORS, MECH_SIZE_TILES};

/// Utilities for mapping between mech interior coordinates and world coordinates
pub struct MechInteriorCoordinates;

impl MechInteriorCoordinates {
    /// Convert mech interior position to world tile position
    /// This allows mech interiors to be rendered in the same world space as outdoor tiles
    pub fn interior_to_world(mech_pos: TilePos, floor: u8, interior_pos: TilePos) -> TilePos {
        // Strategy: Map mech interiors to virtual tile spaces
        // Floor 0: Uses the actual mech's world position as base
        // Floor 1+: Maps to virtual space above the world for rendering purposes

        // Base offset for mech interior positioning
        let base_x = mech_pos.x + interior_pos.x;
        let base_y = mech_pos.y + interior_pos.y;

        // For higher floors, we use virtual Y coordinates to avoid collision with ground tiles
        // This allows the vision system to "see" upper floors when looking through windows
        let virtual_y_offset = floor as i32 * (FLOOR_HEIGHT_TILES + 1); // +1 for separation

        TilePos::new(base_x, base_y + virtual_y_offset)
    }

    /// Convert world position back to mech interior coordinates
    /// Returns None if the world position doesn't correspond to a mech interior
    pub fn world_to_interior(world_pos: TilePos, mech_pos: TilePos) -> Option<(u8, TilePos)> {
        // Check if this world position could be inside this mech
        let relative_x = world_pos.x - mech_pos.x;
        let relative_y_base = world_pos.y - mech_pos.y;

        // Must be within mech bounds horizontally
        if relative_x < 0 || relative_x >= FLOOR_WIDTH_TILES {
            return None;
        }

        // Check each floor to see if this Y coordinate matches
        for floor in 0..MECH_FLOORS as u8 {
            let floor_y_offset = floor as i32 * (FLOOR_HEIGHT_TILES + 1);
            let relative_y = relative_y_base - floor_y_offset;

            if relative_y >= 0 && relative_y < FLOOR_HEIGHT_TILES {
                return Some((floor, TilePos::new(relative_x, relative_y)));
            }
        }

        None
    }

    /// Get all world positions that correspond to a specific mech interior tile
    pub fn get_all_world_positions_for_mech(mech_pos: TilePos) -> Vec<(u8, TilePos, TilePos)> {
        let mut positions = Vec::new();

        for floor in 0..MECH_FLOORS as u8 {
            for y in 0..FLOOR_HEIGHT_TILES {
                for x in 0..FLOOR_WIDTH_TILES {
                    let interior_pos = TilePos::new(x, y);
                    let world_pos = Self::interior_to_world(mech_pos, floor, interior_pos);
                    positions.push((floor, interior_pos, world_pos));
                }
            }
        }

        positions
    }

    /// Check if a world position is a mech door from the outside
    pub fn is_mech_door_from_outside(world_pos: TilePos, mech_pos: TilePos) -> bool {
        let doors = MechDoorPositions::from_mech_position(mech_pos);
        doors.is_door_tile(world_pos)
    }

    /// Check if looking from outside position to inside position crosses a window
    pub fn crosses_window(
        outside_pos: WorldPos,
        mech_pos: TilePos,
        _interior_floor: u8,
        _interior_pos: TilePos,
    ) -> Option<Direction> {
        // Check if the ray from outside to interior would cross a window tile
        // This is simplified - a full implementation would check the actual mech layout for windows

        // For now, assume mechs have windows on their sides
        let mech_world = mech_pos.to_world();
        let mech_center_x = mech_world.x + (MECH_SIZE_TILES as f32 * crate::TILE_SIZE) / 2.0;
        let mech_center_y = mech_world.y + (MECH_SIZE_TILES as f32 * crate::TILE_SIZE) / 2.0;

        // Determine which side of the mech the outside position is on
        let dx = outside_pos.x - mech_center_x;
        let dy = outside_pos.y - mech_center_y;

        if dx.abs() > dy.abs() {
            // Looking from left or right side
            if dx > 0.0 {
                Some(Direction::Right)
            } else {
                Some(Direction::Left)
            }
        } else {
            // Looking from top or bottom
            if dy > 0.0 {
                Some(Direction::Down)
            } else {
                Some(Direction::Up)
            }
        }
    }

    /// Get the world bounds of a mech (all floors combined)
    pub fn get_mech_world_bounds(mech_pos: TilePos) -> (TilePos, TilePos) {
        let top_floor = MECH_FLOORS as u8 - 1;
        let min_pos = Self::interior_to_world(mech_pos, 0, TilePos::new(0, 0));
        let max_pos = Self::interior_to_world(
            mech_pos,
            top_floor,
            TilePos::new(FLOOR_WIDTH_TILES - 1, FLOOR_HEIGHT_TILES - 1),
        );

        (min_pos, max_pos)
    }

    /// Check if a world position is within any floor of a mech
    pub fn is_within_mech_bounds(world_pos: TilePos, mech_pos: TilePos) -> bool {
        Self::world_to_interior(world_pos, mech_pos).is_some()
    }

    /// Get the relative distance between an exterior position and a mech interior tile
    /// This is useful for vision calculations - closer tiles are easier to see through windows
    pub fn distance_to_interior(
        outside_pos: WorldPos,
        mech_pos: TilePos,
        floor: u8,
        interior_pos: TilePos,
    ) -> f32 {
        // Convert interior position to world position for distance calculation
        let interior_world_pos = Self::interior_to_world(mech_pos, floor, interior_pos);
        let interior_world_actual = interior_world_pos.to_world_center();

        outside_pos.distance_to(interior_world_actual)
    }
}

/// Utilities for visibility through mech openings
pub struct MechVisionUtils;

impl MechVisionUtils {
    /// Check if there's a clear line of sight from outside to inside a mech
    /// This considers doors, windows, and floor visibility
    pub fn can_see_into_mech(
        viewer_pos: WorldPos,
        mech_pos: TilePos,
        target_floor: u8,
        target_interior_pos: TilePos,
    ) -> (bool, f32) {
        // Basic distance check
        let distance = MechInteriorCoordinates::distance_to_interior(
            viewer_pos,
            mech_pos,
            target_floor,
            target_interior_pos,
        );

        // Too far away
        if distance > 200.0 {
            // Roughly 6-7 tiles
            return (false, 0.0);
        }

        // Check if we're looking through a door (only floor 0)
        if target_floor == 0 {
            let doors = MechDoorPositions::from_mech_position(mech_pos);
            let viewer_tile = viewer_pos.to_tile();

            // If viewer is near a door and looking at floor 0, good visibility
            for door_pos in doors.door_tiles() {
                if viewer_tile.distance_to(door_pos) <= 2.0 {
                    return (true, 1.0 - (distance / 100.0).min(0.8));
                }
            }
        }

        // Check if we're looking through a window (simplified)
        if let Some(_window_side) = MechInteriorCoordinates::crosses_window(
            viewer_pos,
            mech_pos,
            target_floor,
            target_interior_pos,
        ) {
            // Windows provide limited visibility with distance falloff
            let visibility = (1.0 - (distance / 150.0)).max(0.0) * 0.7; // 70% max visibility through windows
            return (visibility > 0.1, visibility);
        }

        // No clear line of sight
        (false, 0.0)
    }

    /// Get all interior tiles that might be visible from an outside position
    pub fn get_potentially_visible_interior_tiles(
        viewer_pos: WorldPos,
        mech_pos: TilePos,
        max_distance: f32,
    ) -> Vec<(u8, TilePos, f32)> {
        let mut visible_tiles = Vec::new();

        for floor in 0..MECH_FLOORS as u8 {
            for y in 0..FLOOR_HEIGHT_TILES {
                for x in 0..FLOOR_WIDTH_TILES {
                    let interior_pos = TilePos::new(x, y);
                    let (can_see, visibility) =
                        Self::can_see_into_mech(viewer_pos, mech_pos, floor, interior_pos);

                    if can_see && visibility > 0.1 {
                        let distance = MechInteriorCoordinates::distance_to_interior(
                            viewer_pos,
                            mech_pos,
                            floor,
                            interior_pos,
                        );

                        if distance <= max_distance {
                            visible_tiles.push((floor, interior_pos, visibility));
                        }
                    }
                }
            }
        }

        visible_tiles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TILE_SIZE;

    #[test]
    fn test_interior_to_world_conversion() {
        let mech_pos = TilePos::new(10, 10);
        let interior_pos = TilePos::new(2, 3);

        // Floor 0 should map directly
        let world_pos_floor_0 =
            MechInteriorCoordinates::interior_to_world(mech_pos, 0, interior_pos);
        assert_eq!(world_pos_floor_0, TilePos::new(12, 13));

        // Floor 1 should be offset vertically
        let world_pos_floor_1 =
            MechInteriorCoordinates::interior_to_world(mech_pos, 1, interior_pos);
        assert_eq!(
            world_pos_floor_1,
            TilePos::new(12, 13 + FLOOR_HEIGHT_TILES + 1)
        );
    }

    #[test]
    fn test_world_to_interior_conversion() {
        let mech_pos = TilePos::new(10, 10);
        let world_pos = TilePos::new(12, 13);

        // Should map back to floor 0
        let interior = MechInteriorCoordinates::world_to_interior(world_pos, mech_pos);
        assert_eq!(interior, Some((0, TilePos::new(2, 3))));

        // Test higher floor
        let world_pos_floor_1 = TilePos::new(12, 13 + FLOOR_HEIGHT_TILES + 1);
        let interior_floor_1 =
            MechInteriorCoordinates::world_to_interior(world_pos_floor_1, mech_pos);
        assert_eq!(interior_floor_1, Some((1, TilePos::new(2, 3))));
    }

    #[test]
    fn test_mech_bounds_checking() {
        let mech_pos = TilePos::new(10, 10);

        // Inside bounds
        let inside_pos = TilePos::new(11, 11);
        assert!(MechInteriorCoordinates::is_within_mech_bounds(
            inside_pos, mech_pos
        ));

        // Outside bounds
        let outside_pos = TilePos::new(5, 5);
        assert!(!MechInteriorCoordinates::is_within_mech_bounds(
            outside_pos,
            mech_pos
        ));
    }

    #[test]
    fn test_door_detection() {
        let mech_pos = TilePos::new(10, 10);
        let doors = MechDoorPositions::from_mech_position(mech_pos);

        // Should detect door tiles
        assert!(MechInteriorCoordinates::is_mech_door_from_outside(
            doors.left_door,
            mech_pos
        ));
        assert!(MechInteriorCoordinates::is_mech_door_from_outside(
            doors.right_door,
            mech_pos
        ));

        // Should not detect non-door tiles
        assert!(!MechInteriorCoordinates::is_mech_door_from_outside(
            TilePos::new(0, 0),
            mech_pos
        ));
    }

    #[test]
    fn test_vision_into_mech() {
        let mech_pos = TilePos::new(10, 10);
        let viewer_pos = WorldPos::new(8.0 * TILE_SIZE, 10.0 * TILE_SIZE); // Left side of mech
        let interior_pos = TilePos::new(1, 1);

        // Should be able to see into floor 0 from nearby
        let (can_see, visibility) =
            MechVisionUtils::can_see_into_mech(viewer_pos, mech_pos, 0, interior_pos);

        assert!(can_see);
        assert!(visibility > 0.0);

        // Should not see from very far away
        let far_viewer_pos = WorldPos::new(0.0, 0.0);
        let (can_see_far, _) =
            MechVisionUtils::can_see_into_mech(far_viewer_pos, mech_pos, 0, interior_pos);

        assert!(!can_see_far);
    }
}
