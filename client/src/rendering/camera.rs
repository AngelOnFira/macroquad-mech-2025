use macroquad::prelude::*;
use shared::{
    constants::{ARENA_HEIGHT_TILES, ARENA_WIDTH_TILES, TILE_SIZE},
    coordinates::{ScreenPos, TilePos, TileRegion, WorldPos},
};

/// Camera transformation utilities for converting between coordinate spaces
pub struct Camera {
    pub position: WorldPos,
    pub zoom: f32,
}

impl Camera {
    /// Create a new camera at the given world position
    pub fn new(position: WorldPos) -> Self {
        Self {
            position,
            zoom: 1.0,
        }
    }

    /// Create a camera centered on a tile
    pub fn centered_on_tile(tile_pos: TilePos) -> Self {
        Self::new(tile_pos.to_world_center())
    }

    /// Get the camera offset for rendering (negated position)
    pub fn get_offset(&self) -> WorldPos {
        WorldPos::new(-self.position.x, -self.position.y)
    }

    /// Convert world position to screen position
    pub fn world_to_screen(&self, world_pos: WorldPos) -> ScreenPos {
        let offset = self.get_offset();
        ScreenPos::new(
            (world_pos.x + offset.x) * self.zoom + screen_width() / 2.0,
            (world_pos.y + offset.y) * self.zoom + screen_height() / 2.0,
        )
    }

    /// Convert screen position to world position
    pub fn screen_to_world(&self, screen_pos: ScreenPos) -> WorldPos {
        let offset = self.get_offset();
        WorldPos::new(
            (screen_pos.x - screen_width() / 2.0) / self.zoom - offset.x,
            (screen_pos.y - screen_height() / 2.0) / self.zoom - offset.y,
        )
    }

    /// Convert tile position directly to screen position
    pub fn tile_to_screen(&self, tile_pos: TilePos) -> ScreenPos {
        let world_pos = tile_pos.to_world();
        self.world_to_screen(world_pos)
    }

    /// Convert screen position to tile position
    pub fn screen_to_tile(&self, screen_pos: ScreenPos) -> TilePos {
        let world_pos = self.screen_to_world(screen_pos);
        world_pos.to_tile()
    }

    /// Move the camera by a given offset
    pub fn move_by(&mut self, offset: WorldPos) {
        self.position = self.position + offset;
    }

    /// Set the camera position to follow a target with optional smoothing
    pub fn follow(&mut self, target: WorldPos, smoothing: f32) {
        if smoothing <= 0.0 {
            self.position = target;
        } else {
            self.position = self.position.lerp(target, smoothing);
        }
    }

    /// Clamp the camera position to stay within world bounds
    pub fn clamp_to_world(&mut self) {
        let half_screen_width = screen_width() / 2.0 / self.zoom;
        let half_screen_height = screen_height() / 2.0 / self.zoom;

        let world_width = ARENA_WIDTH_TILES as f32 * TILE_SIZE;
        let world_height = ARENA_HEIGHT_TILES as f32 * TILE_SIZE;

        self.position.x = self
            .position
            .x
            .max(half_screen_width)
            .min(world_width - half_screen_width);
        self.position.y = self
            .position
            .y
            .max(half_screen_height)
            .min(world_height - half_screen_height);
    }

    /// Set the zoom level (clamped to reasonable values)
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 10.0);
    }

    /// Zoom in by a factor
    pub fn zoom_in(&mut self, factor: f32) {
        self.set_zoom(self.zoom * factor);
    }

    /// Zoom out by a factor
    pub fn zoom_out(&mut self, factor: f32) {
        self.set_zoom(self.zoom / factor);
    }
}

/// Utilities for calculating visible regions and culling
pub struct ViewportCalculations;

impl ViewportCalculations {
    /// Calculate which tiles are visible on screen
    pub fn get_visible_tile_range(camera: &Camera) -> TileRegion {
        let half_screen_width = screen_width() / 2.0 / camera.zoom;
        let half_screen_height = screen_height() / 2.0 / camera.zoom;

        let min_world = WorldPos::new(
            camera.position.x - half_screen_width,
            camera.position.y - half_screen_height,
        );
        let max_world = WorldPos::new(
            camera.position.x + half_screen_width,
            camera.position.y + half_screen_height,
        );

        let min_tile = min_world.to_tile();
        let max_tile = max_world.to_tile();

        // Add padding to ensure we don't miss tiles at the edges
        TileRegion::new(
            TilePos::new(min_tile.x - 1, min_tile.y - 1),
            TilePos::new(max_tile.x + 1, max_tile.y + 1),
        )
    }

    /// Calculate visible tile range with custom padding
    pub fn get_visible_tile_range_with_padding(camera: &Camera, padding_tiles: i32) -> TileRegion {
        let base_range = Self::get_visible_tile_range(camera);
        TileRegion::new(
            TilePos::new(
                base_range.min.x - padding_tiles,
                base_range.min.y - padding_tiles,
            ),
            TilePos::new(
                base_range.max.x + padding_tiles,
                base_range.max.y + padding_tiles,
            ),
        )
    }

    /// Check if a world position is visible on screen
    pub fn is_world_pos_visible(camera: &Camera, world_pos: WorldPos) -> bool {
        let screen_pos = camera.world_to_screen(world_pos);
        screen_pos.x >= 0.0
            && screen_pos.x <= screen_width()
            && screen_pos.y >= 0.0
            && screen_pos.y <= screen_height()
    }

    /// Check if a tile is visible on screen
    pub fn is_tile_visible(camera: &Camera, tile_pos: TilePos) -> bool {
        let visible_range = Self::get_visible_tile_range(camera);
        visible_range.contains(tile_pos)
    }

    /// Check if a rectangular area is visible (for entity culling)
    pub fn is_rect_visible(camera: &Camera, world_pos: WorldPos, width: f32, height: f32) -> bool {
        let screen_pos = camera.world_to_screen(world_pos);
        let screen_width_scaled = width * camera.zoom;
        let screen_height_scaled = height * camera.zoom;

        // Check if rectangle overlaps with screen bounds
        screen_pos.x + screen_width_scaled >= 0.0
            && screen_pos.x <= screen_width()
            && screen_pos.y + screen_height_scaled >= 0.0
            && screen_pos.y <= screen_height()
    }

    /// Get the world bounds of what's currently visible on screen
    pub fn get_visible_world_bounds(camera: &Camera) -> (WorldPos, WorldPos) {
        let min_screen = ScreenPos::new(0.0, 0.0);
        let max_screen = ScreenPos::new(screen_width(), screen_height());

        let min_world = camera.screen_to_world(min_screen);
        let max_world = camera.screen_to_world(max_screen);

        (min_world, max_world)
    }
}

/// Camera behavior presets for common use cases
pub struct CameraBehavior;

impl CameraBehavior {
    /// Smooth camera following with deadzone
    pub fn smooth_follow_with_deadzone(
        camera: &mut Camera,
        target: WorldPos,
        deadzone_radius: f32,
        follow_speed: f32,
    ) {
        let distance = camera.position.distance_to(target);
        if distance > deadzone_radius {
            let direction = camera.position.direction_to(target);
            let move_distance = (distance - deadzone_radius) * follow_speed;
            let offset = direction * move_distance;
            camera.move_by(offset);
        }
    }

    /// Snap camera to follow target with grid alignment
    pub fn snap_follow_grid_aligned(camera: &mut Camera, target: WorldPos, grid_size: f32) {
        let aligned_target = WorldPos::new(
            (target.x / grid_size).round() * grid_size,
            (target.y / grid_size).round() * grid_size,
        );
        camera.position = aligned_target;
    }

    /// Predict target movement for smooth camera following
    pub fn predictive_follow(
        camera: &mut Camera,
        target: WorldPos,
        target_velocity: WorldPos,
        prediction_time: f32,
        follow_speed: f32,
    ) {
        let predicted_target = WorldPos::new(
            target.x + target_velocity.x * prediction_time,
            target.y + target_velocity.y * prediction_time,
        );

        camera.follow(predicted_target, follow_speed);
    }

    /// Keep camera within world bounds while following target
    pub fn constrained_follow(
        camera: &mut Camera,
        target: WorldPos,
        follow_speed: f32,
        world_bounds: Option<TileRegion>,
    ) {
        camera.follow(target, follow_speed);

        if world_bounds.is_some() {
            camera.clamp_to_world();
        }
    }
}

/// Screen space utilities
pub struct ScreenSpace;

impl ScreenSpace {
    /// Convert mouse position to world position
    pub fn mouse_to_world(camera: &Camera) -> WorldPos {
        let mouse_pos = mouse_position();
        let screen_pos = ScreenPos::new(mouse_pos.0, mouse_pos.1);
        camera.screen_to_world(screen_pos)
    }

    /// Convert mouse position to tile position
    pub fn mouse_to_tile(camera: &Camera) -> TilePos {
        let world_pos = Self::mouse_to_world(camera);
        world_pos.to_tile()
    }

    /// Check if mouse is over a specific tile
    pub fn is_mouse_over_tile(camera: &Camera, tile_pos: TilePos) -> bool {
        let mouse_tile = Self::mouse_to_tile(camera);
        mouse_tile == tile_pos
    }

    /// Check if mouse is over a specific world area
    pub fn is_mouse_over_area(
        camera: &Camera,
        world_pos: WorldPos,
        width: f32,
        height: f32,
    ) -> bool {
        let mouse_world = Self::mouse_to_world(camera);
        mouse_world.x >= world_pos.x
            && mouse_world.x <= world_pos.x + width
            && mouse_world.y >= world_pos.y
            && mouse_world.y <= world_pos.y + height
    }

    /// Get the tile the mouse is currently over
    pub fn get_hovered_tile(camera: &Camera) -> Option<TilePos> {
        let tile_pos = Self::mouse_to_tile(camera);

        // Check if the tile is within world bounds
        if tile_pos.is_in_world_bounds() {
            Some(tile_pos)
        } else {
            None
        }
    }
}

/// Camera shake effects
pub struct CameraShake {
    intensity: f32,
    duration: f32,
    remaining_time: f32,
    offset: WorldPos,
}

impl CameraShake {
    pub fn new(intensity: f32, duration: f32) -> Self {
        Self {
            intensity,
            duration,
            remaining_time: duration,
            offset: WorldPos::new(0.0, 0.0),
        }
    }

    /// Update the shake effect
    pub fn update(&mut self, delta_time: f32) {
        if self.remaining_time > 0.0 {
            self.remaining_time -= delta_time;

            let shake_factor = self.remaining_time / self.duration;
            let current_intensity = self.intensity * shake_factor;

            // Generate random offset
            use macroquad::rand::gen_range;
            self.offset = WorldPos::new(
                gen_range(-current_intensity, current_intensity),
                gen_range(-current_intensity, current_intensity),
            );
        } else {
            self.offset = WorldPos::new(0.0, 0.0);
        }
    }

    /// Apply shake offset to a camera
    pub fn apply_to_camera(&self, camera: &mut Camera) {
        camera.position = camera.position + self.offset;
    }

    /// Check if shake is still active
    pub fn is_active(&self) -> bool {
        self.remaining_time > 0.0
    }

    /// Reset the shake effect
    pub fn reset(&mut self, intensity: f32, duration: f32) {
        self.intensity = intensity;
        self.duration = duration;
        self.remaining_time = duration;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new(WorldPos::new(100.0, 200.0));
        assert_eq!(camera.position.x, 100.0);
        assert_eq!(camera.position.y, 200.0);
        assert_eq!(camera.zoom, 1.0);
    }

    #[test]
    fn test_camera_offset() {
        let camera = Camera::new(WorldPos::new(100.0, 200.0));
        let offset = camera.get_offset();
        assert_eq!(offset.x, -100.0);
        assert_eq!(offset.y, -200.0);
    }

    #[test]
    fn test_camera_movement() {
        let mut camera = Camera::new(WorldPos::new(0.0, 0.0));
        camera.move_by(WorldPos::new(50.0, 75.0));
        assert_eq!(camera.position.x, 50.0);
        assert_eq!(camera.position.y, 75.0);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera::new(WorldPos::new(0.0, 0.0));
        camera.set_zoom(2.0);
        assert_eq!(camera.zoom, 2.0);

        camera.zoom_in(1.5);
        assert_eq!(camera.zoom, 3.0);

        camera.zoom_out(3.0);
        assert_eq!(camera.zoom, 1.0);
    }

    #[test]
    fn test_camera_follow() {
        let mut camera = Camera::new(WorldPos::new(0.0, 0.0));
        let target = WorldPos::new(100.0, 100.0);

        camera.follow(target, 0.5);
        assert_eq!(camera.position.x, 50.0);
        assert_eq!(camera.position.y, 50.0);

        camera.follow(target, 0.0); // No smoothing
        assert_eq!(camera.position, target);
    }

    #[test]
    fn test_camera_shake() {
        let mut shake = CameraShake::new(10.0, 1.0);
        assert!(shake.is_active());

        shake.update(0.5); // Update halfway through
        assert!(shake.is_active());
        assert!(shake.offset.magnitude() > 0.0);

        shake.update(1.0); // Update past duration
        assert!(!shake.is_active());
        assert_eq!(shake.offset.magnitude(), 0.0);
    }
}
