use macroquad::prelude::*;
use shared::{
    types::Direction, 
    coordinates::{WorldPos, TilePos, RelativePosition},
    constants::TILE_SIZE,
};

/// Style configuration for drawing arrows
#[derive(Debug, Clone, Copy)]
pub struct ArrowStyle {
    pub color: Color,
    pub size_ratio: f32,    // Size as ratio of tile size (0.0-1.0)
    pub line_thickness: f32,
    pub filled: bool,
}

impl Default for ArrowStyle {
    fn default() -> Self {
        Self {
            color: Color::from_rgba(200, 200, 210, 150),
            size_ratio: 0.3,
            line_thickness: 2.0,
            filled: true,
        }
    }
}

impl ArrowStyle {
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    
    pub fn with_size_ratio(mut self, size_ratio: f32) -> Self {
        self.size_ratio = size_ratio.clamp(0.0, 1.0);
        self
    }
    
    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.line_thickness = thickness;
        self
    }
    
    pub fn outlined(mut self) -> Self {
        self.filled = false;
        self
    }
    
    pub fn filled(mut self) -> Self {
        self.filled = true;
        self
    }
}

/// Arrow drawing utilities
pub struct ArrowRenderer;

impl ArrowRenderer {
    /// Draw an arrow at a world position
    pub fn draw_arrow_at_world(pos: WorldPos, direction: Direction, size: f32, style: ArrowStyle) {
        let center_x = pos.x;
        let center_y = pos.y;
        let arrow_size = size * style.size_ratio;
        
        Self::draw_arrow_triangles(center_x, center_y, arrow_size, direction, style);
    }
    
    /// Draw an arrow in a tile at a specific relative position
    pub fn draw_arrow_in_tile(tile: TilePos, relative_pos: RelativePosition, direction: Direction, style: ArrowStyle) {
        let world_pos = relative_pos.world_pos_in_tile(tile);
        Self::draw_arrow_at_world(world_pos, direction, TILE_SIZE, style);
    }
    
    /// Draw an arrow at the center of a tile
    pub fn draw_arrow_centered_in_tile(tile: TilePos, direction: Direction, style: ArrowStyle) {
        Self::draw_arrow_in_tile(tile, RelativePosition::Center, direction, style);
    }
    
    /// Draw an arrow with screen coordinates
    pub fn draw_arrow_at_screen(screen_x: f32, screen_y: f32, size: f32, direction: Direction, style: ArrowStyle) {
        let arrow_size = size * style.size_ratio;
        Self::draw_arrow_triangles(screen_x, screen_y, arrow_size, direction, style);
    }
    
    /// Internal function to draw arrow triangles
    fn draw_arrow_triangles(center_x: f32, center_y: f32, arrow_size: f32, direction: Direction, style: ArrowStyle) {
        match direction {
            Direction::Up => {
                let p1 = Vec2::new(center_x, center_y - arrow_size);
                let p2 = Vec2::new(center_x - arrow_size / 2.0, center_y);
                let p3 = Vec2::new(center_x + arrow_size / 2.0, center_y);
                
                if style.filled {
                    draw_triangle(p1, p2, p3, style.color);
                } else {
                    draw_triangle_lines(p1, p2, p3, style.line_thickness, style.color);
                }
            }
            Direction::Down => {
                let p1 = Vec2::new(center_x, center_y + arrow_size);
                let p2 = Vec2::new(center_x - arrow_size / 2.0, center_y);
                let p3 = Vec2::new(center_x + arrow_size / 2.0, center_y);
                
                if style.filled {
                    draw_triangle(p1, p2, p3, style.color);
                } else {
                    draw_triangle_lines(p1, p2, p3, style.line_thickness, style.color);
                }
            }
            Direction::Left => {
                let p1 = Vec2::new(center_x - arrow_size, center_y);
                let p2 = Vec2::new(center_x, center_y - arrow_size / 2.0);
                let p3 = Vec2::new(center_x, center_y + arrow_size / 2.0);
                
                if style.filled {
                    draw_triangle(p1, p2, p3, style.color);
                } else {
                    draw_triangle_lines(p1, p2, p3, style.line_thickness, style.color);
                }
            }
            Direction::Right => {
                let p1 = Vec2::new(center_x + arrow_size, center_y);
                let p2 = Vec2::new(center_x, center_y - arrow_size / 2.0);
                let p3 = Vec2::new(center_x, center_y + arrow_size / 2.0);
                
                if style.filled {
                    draw_triangle(p1, p2, p3, style.color);
                } else {
                    draw_triangle_lines(p1, p2, p3, style.line_thickness, style.color);
                }
            }
        }
    }
}

/// Style configuration for tile highlights
#[derive(Debug, Clone, Copy)]
pub struct TileHighlightStyle {
    pub color: Color,
    pub border_color: Color,
    pub border_thickness: f32,
    pub corner_radius: f32,
    pub filled: bool,
}

impl Default for TileHighlightStyle {
    fn default() -> Self {
        Self {
            color: Color::from_rgba(255, 255, 255, 50),
            border_color: WHITE,
            border_thickness: 2.0,
            corner_radius: 0.0,
            filled: true,
        }
    }
}

impl TileHighlightStyle {
    pub fn selection() -> Self {
        Self {
            color: Color::from_rgba(0, 255, 0, 100),
            border_color: GREEN,
            border_thickness: 3.0,
            corner_radius: 4.0,
            filled: true,
        }
    }
    
    pub fn hover() -> Self {
        Self {
            color: Color::from_rgba(255, 255, 0, 80),
            border_color: YELLOW,
            border_thickness: 2.0,
            corner_radius: 2.0,
            filled: true,
        }
    }
    
    pub fn danger() -> Self {
        Self {
            color: Color::from_rgba(255, 0, 0, 120),
            border_color: RED,
            border_thickness: 2.0,
            corner_radius: 0.0,
            filled: true,
        }
    }
}

/// Tile highlighting utilities
pub struct TileHighlight;

impl TileHighlight {
    /// Highlight a tile at screen coordinates
    pub fn draw_at_screen(screen_x: f32, screen_y: f32, size: f32, style: TileHighlightStyle) {
        if style.filled {
            if style.corner_radius > 0.0 {
                // TODO: Macroquad doesn't have rounded rectangles, use regular rectangle
                draw_rectangle(screen_x, screen_y, size, size, style.color);
            } else {
                draw_rectangle(screen_x, screen_y, size, size, style.color);
            }
        }
        
        if style.border_thickness > 0.0 {
            draw_rectangle_lines(
                screen_x, 
                screen_y, 
                size, 
                size, 
                style.border_thickness, 
                style.border_color
            );
        }
    }
    
    /// Highlight a tile at world position (requires camera offset)
    pub fn draw_at_world(world_pos: WorldPos, size: f32, camera_offset: WorldPos, style: TileHighlightStyle) {
        let screen_x = world_pos.x + camera_offset.x;
        let screen_y = world_pos.y + camera_offset.y;
        Self::draw_at_screen(screen_x, screen_y, size, style);
    }
    
    /// Highlight a tile position (requires camera offset)
    pub fn draw_tile(tile_pos: TilePos, camera_offset: WorldPos, style: TileHighlightStyle) {
        let world_pos = tile_pos.to_world();
        Self::draw_at_world(world_pos, TILE_SIZE, camera_offset, style);
    }
    
    /// Highlight multiple tiles
    pub fn draw_tiles(tile_positions: &[TilePos], camera_offset: WorldPos, style: TileHighlightStyle) {
        for &tile_pos in tile_positions {
            Self::draw_tile(tile_pos, camera_offset, style);
        }
    }
}

/// Grid rendering utilities
pub struct GridRenderer;

impl GridRenderer {
    /// Draw a tile grid over a region
    pub fn draw_tile_grid(
        min_world: WorldPos,
        max_world: WorldPos,
        camera_offset: WorldPos,
        grid_color: Color,
        line_thickness: f32,
    ) {
        let screen_min_x = min_world.x + camera_offset.x;
        let screen_min_y = min_world.y + camera_offset.y;
        let screen_max_x = max_world.x + camera_offset.x;
        let screen_max_y = max_world.y + camera_offset.y;
        
        // Draw vertical lines
        let mut x = screen_min_x;
        while x <= screen_max_x {
            draw_line(x, screen_min_y, x, screen_max_y, line_thickness, grid_color);
            x += TILE_SIZE;
        }
        
        // Draw horizontal lines
        let mut y = screen_min_y;
        while y <= screen_max_y {
            draw_line(screen_min_x, y, screen_max_x, y, line_thickness, grid_color);
            y += TILE_SIZE;
        }
    }
    
    /// Draw a tile grid that covers the entire screen
    pub fn draw_screen_grid(camera_offset: WorldPos, grid_color: Color, line_thickness: f32) {
        let screen_w = screen_width();
        let screen_h = screen_height();
        
        // Convert screen bounds to world coordinates
        let world_min = WorldPos::new(-camera_offset.x, -camera_offset.y);
        let world_max = WorldPos::new(-camera_offset.x + screen_w, -camera_offset.y + screen_h);
        
        // Snap to tile boundaries
        let tile_min = world_min.to_tile();
        let tile_max = world_max.to_tile();
        let world_min_snapped = tile_min.to_world();
        let world_max_snapped = WorldPos::new(
            (tile_max.x + 1) as f32 * TILE_SIZE,
            (tile_max.y + 1) as f32 * TILE_SIZE,
        );
        
        Self::draw_tile_grid(world_min_snapped, world_max_snapped, camera_offset, grid_color, line_thickness);
    }
}

/// Door indicator rendering
pub struct DoorRenderer;

impl DoorRenderer {
    /// Draw a door indicator at screen coordinates
    pub fn draw_door_at_screen(screen_x: f32, screen_y: f32, size: f32, team_color: Color) {
        // Door background (darker than mech)
        let door_color = Color::new(
            team_color.r * 0.3, 
            team_color.g * 0.3, 
            team_color.b * 0.3, 
            1.0
        );
        draw_rectangle(screen_x, screen_y, size, size, door_color);
        
        // Door outline
        draw_rectangle_lines(screen_x, screen_y, size, size, 2.0, WHITE);
        
        // Entry indicator arrow
        let arrow_style = ArrowStyle {
            color: Color::new(1.0, 1.0, 1.0, 0.5),
            size_ratio: 0.4,
            line_thickness: 2.0,
            filled: false,
        };
        
        ArrowRenderer::draw_arrow_at_screen(
            screen_x + size / 2.0,
            screen_y + size / 2.0,
            size,
            Direction::Down,
            arrow_style,
        );
    }
    
    /// Draw a door indicator at a world position
    pub fn draw_door_at_world(world_pos: WorldPos, team_color: Color, camera_offset: WorldPos) {
        let screen_x = world_pos.x + camera_offset.x;
        let screen_y = world_pos.y + camera_offset.y;
        Self::draw_door_at_screen(screen_x, screen_y, TILE_SIZE, team_color);
    }
    
    /// Draw a door indicator at a tile position
    pub fn draw_door_tile(tile_pos: TilePos, team_color: Color, camera_offset: WorldPos) {
        let world_pos = tile_pos.to_world();
        Self::draw_door_at_world(world_pos, team_color, camera_offset);
    }
}

/// Entity positioning within tiles
pub struct EntityRenderer;

impl EntityRenderer {
    /// Draw a circle entity positioned within a tile
    pub fn draw_circle_in_tile(
        tile_pos: TilePos,
        relative_pos: RelativePosition,
        radius: f32,
        color: Color,
        camera_offset: WorldPos,
    ) {
        let world_pos = relative_pos.world_pos_in_tile(tile_pos);
        let screen_x = world_pos.x + camera_offset.x;
        let screen_y = world_pos.y + camera_offset.y;
        draw_circle(screen_x, screen_y, radius, color);
    }
    
    /// Draw a rectangle entity positioned within a tile
    pub fn draw_rect_in_tile(
        tile_pos: TilePos,
        relative_pos: RelativePosition,
        size: (f32, f32),
        color: Color,
        camera_offset: WorldPos,
    ) {
        let world_pos = relative_pos.world_pos_in_tile(tile_pos);
        let screen_x = world_pos.x + camera_offset.x - size.0 / 2.0;
        let screen_y = world_pos.y + camera_offset.y - size.1 / 2.0;
        draw_rectangle(screen_x, screen_y, size.0, size.1, color);
    }
    
    /// Draw a player-like entity (circle with optional direction arrow)
    pub fn draw_player_entity(
        world_pos: WorldPos,
        radius: f32,
        color: Color,
        camera_offset: WorldPos,
        facing_direction: Option<Direction>,
    ) {
        let screen_x = world_pos.x + camera_offset.x;
        let screen_y = world_pos.y + camera_offset.y;
        
        // Draw player circle
        draw_circle(screen_x, screen_y, radius, color);
        
        // Draw direction indicator if specified
        if let Some(direction) = facing_direction {
            let arrow_style = ArrowStyle::default()
                .with_color(Color::new(0.0, 0.0, 0.0, 0.8))
                .with_size_ratio(0.6)
                .outlined();
            
            ArrowRenderer::draw_arrow_at_screen(
                screen_x, screen_y, radius * 2.0, direction, arrow_style
            );
        }
    }
}

/// Common shape utilities
pub struct ShapeRenderer;

impl ShapeRenderer {
    /// Draw a plus sign (+) for debugging or UI
    pub fn draw_plus(center: WorldPos, size: f32, thickness: f32, color: Color, camera_offset: WorldPos) {
        let screen_x = center.x + camera_offset.x;
        let screen_y = center.y + camera_offset.y;
        let half_size = size / 2.0;
        
        // Horizontal line
        draw_line(
            screen_x - half_size, screen_y,
            screen_x + half_size, screen_y,
            thickness, color
        );
        
        // Vertical line
        draw_line(
            screen_x, screen_y - half_size,
            screen_x, screen_y + half_size,
            thickness, color
        );
    }
    
    /// Draw an X mark for debugging or UI
    pub fn draw_x_mark(center: WorldPos, size: f32, thickness: f32, color: Color, camera_offset: WorldPos) {
        let screen_x = center.x + camera_offset.x;
        let screen_y = center.y + camera_offset.y;
        let half_size = size / 2.0;
        
        // Diagonal lines
        draw_line(
            screen_x - half_size, screen_y - half_size,
            screen_x + half_size, screen_y + half_size,
            thickness, color
        );
        draw_line(
            screen_x - half_size, screen_y + half_size,
            screen_x + half_size, screen_y - half_size,
            thickness, color
        );
    }
    
    /// Draw a diamond shape
    pub fn draw_diamond(center: WorldPos, size: f32, color: Color, camera_offset: WorldPos, filled: bool) {
        let screen_x = center.x + camera_offset.x;
        let screen_y = center.y + camera_offset.y;
        let half_size = size / 2.0;
        
        let top = Vec2::new(screen_x, screen_y - half_size);
        let right = Vec2::new(screen_x + half_size, screen_y);
        let bottom = Vec2::new(screen_x, screen_y + half_size);
        let left = Vec2::new(screen_x - half_size, screen_y);
        
        if filled {
            // Draw as two triangles
            draw_triangle(top, right, bottom, color);
            draw_triangle(top, bottom, left, color);
        } else {
            // Draw as lines
            let thickness = 2.0;
            draw_line(top.x, top.y, right.x, right.y, thickness, color);
            draw_line(right.x, right.y, bottom.x, bottom.y, thickness, color);
            draw_line(bottom.x, bottom.y, left.x, left.y, thickness, color);
            draw_line(left.x, left.y, top.x, top.y, thickness, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arrow_style() {
        let style = ArrowStyle::default()
            .with_color(RED)
            .with_size_ratio(0.5)
            .with_thickness(3.0)
            .outlined();
        
        assert_eq!(style.color, RED);
        assert_eq!(style.size_ratio, 0.5);
        assert_eq!(style.line_thickness, 3.0);
        assert!(!style.filled);
    }
    
    #[test]
    fn test_tile_highlight_styles() {
        let selection = TileHighlightStyle::selection();
        assert_eq!(selection.border_color, GREEN);
        
        let hover = TileHighlightStyle::hover();
        assert_eq!(hover.border_color, YELLOW);
        
        let danger = TileHighlightStyle::danger();
        assert_eq!(danger.border_color, RED);
    }
}