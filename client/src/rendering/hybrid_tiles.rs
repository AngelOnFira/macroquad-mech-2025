use macroquad::prelude::*;
use shared::{TileVisual, Material, Direction, StationType, TILE_SIZE};

/// Render a tile using the hybrid tile visual system
pub fn render_tile_visual(tile: &TileVisual, x: f32, y: f32, size: f32) {
    match tile {
        TileVisual::Floor { material, wear } => {
            let base_color = match material {
                Material::Metal => Color::from_rgba(100, 100, 110, 255),
                Material::Reinforced => Color::from_rgba(80, 80, 90, 255),
                Material::Damaged => Color::from_rgba(60, 50, 50, 255),
            };
            
            // Apply wear
            let wear_factor = 1.0 - (*wear as f32 / 255.0) * 0.3;
            let color = Color::new(
                base_color.r * wear_factor,
                base_color.g * wear_factor,
                base_color.b * wear_factor,
                base_color.a,
            );
            
            draw_rectangle(x, y, size, size, color);
            
            // Draw grid lines for floor tiles
            draw_rectangle_lines(x, y, size, size, 1.0, Color::from_rgba(60, 60, 70, 100));
        }
        
        TileVisual::Wall { material } => {
            let color = match material {
                Material::Metal => Color::from_rgba(50, 50, 60, 255),
                Material::Reinforced => Color::from_rgba(30, 30, 40, 255),
                Material::Damaged => Color::from_rgba(40, 30, 30, 255),
            };
            
            draw_rectangle(x, y, size, size, color);
            
            // Draw highlight on top edge
            draw_line(x, y, x + size, y, 2.0, Color::from_rgba(80, 80, 90, 255));
        }
        
        TileVisual::Window { broken, facing } => {
            // Draw window frame
            draw_rectangle(x, y, size, size, Color::from_rgba(40, 40, 50, 255));
            
            if *broken {
                // Draw broken glass effect
                draw_line(x, y, x + size, y + size, 2.0, Color::from_rgba(150, 150, 160, 200));
                draw_line(x + size, y, x, y + size, 2.0, Color::from_rgba(150, 150, 160, 200));
            } else {
                // Draw glass
                let glass_color = Color::from_rgba(100, 120, 140, 100);
                draw_rectangle(x + 2.0, y + 2.0, size - 4.0, size - 4.0, glass_color);
                
                // Draw directional indicator
                let center_x = x + size / 2.0;
                let center_y = y + size / 2.0;
                let arrow_size = size * 0.3;
                
                match facing {
                    Direction::Up => {
                        draw_triangle(
                            Vec2::new(center_x, center_y - arrow_size),
                            Vec2::new(center_x - arrow_size/2.0, center_y),
                            Vec2::new(center_x + arrow_size/2.0, center_y),
                            Color::from_rgba(200, 200, 210, 150),
                        );
                    }
                    Direction::Down => {
                        draw_triangle(
                            Vec2::new(center_x, center_y + arrow_size),
                            Vec2::new(center_x - arrow_size/2.0, center_y),
                            Vec2::new(center_x + arrow_size/2.0, center_y),
                            Color::from_rgba(200, 200, 210, 150),
                        );
                    }
                    Direction::Left => {
                        draw_triangle(
                            Vec2::new(center_x - arrow_size, center_y),
                            Vec2::new(center_x, center_y - arrow_size/2.0),
                            Vec2::new(center_x, center_y + arrow_size/2.0),
                            Color::from_rgba(200, 200, 210, 150),
                        );
                    }
                    Direction::Right => {
                        draw_triangle(
                            Vec2::new(center_x + arrow_size, center_y),
                            Vec2::new(center_x, center_y - arrow_size/2.0),
                            Vec2::new(center_x, center_y + arrow_size/2.0),
                            Color::from_rgba(200, 200, 210, 150),
                        );
                    }
                }
            }
        }
        
        TileVisual::Station { station_type, active } => {
            // Draw floor first
            draw_rectangle(x, y, size, size, Color::from_rgba(100, 100, 110, 255));
            
            // Draw station
            let station_color = if *active {
                Color::from_rgba(100, 255, 100, 255)
            } else {
                Color::from_rgba(150, 150, 160, 255)
            };
            
            let station_size = size * 0.8;
            let offset = size * 0.1;
            draw_rectangle(x + offset, y + offset, station_size, station_size, station_color);
            
            // Draw station type indicator
            let text = match station_type {
                StationType::WeaponLaser => "L",
                StationType::WeaponProjectile => "P",
                StationType::Engine => "E",
                StationType::Shield => "S",
                StationType::Repair => "R",
                StationType::Electrical => "⚡",
                StationType::Upgrade => "U",
                StationType::Pilot => "◎",
            };
            
            let text_size = size * 0.4;
            draw_text(
                text,
                x + size / 2.0 - text_size / 2.0,
                y + size / 2.0 + text_size / 3.0,
                text_size,
                BLACK,
            );
        }
        
        TileVisual::Turret { facing, firing } => {
            // Draw base
            draw_rectangle(x, y, size, size, Color::from_rgba(60, 60, 70, 255));
            
            // Draw turret
            let turret_color = if *firing {
                Color::from_rgba(255, 100, 100, 255)
            } else {
                Color::from_rgba(120, 120, 130, 255)
            };
            
            let center_x = x + size / 2.0;
            let center_y = y + size / 2.0;
            let turret_radius = size * 0.3;
            
            draw_circle(center_x, center_y, turret_radius, turret_color);
            
            // Draw barrel
            let barrel_length = size * 0.4;
            let (dx, dy) = match facing {
                Direction::Up => (0.0, -1.0),
                Direction::Down => (0.0, 1.0),
                Direction::Left => (-1.0, 0.0),
                Direction::Right => (1.0, 0.0),
            };
            
            draw_line(
                center_x,
                center_y,
                center_x + dx * barrel_length,
                center_y + dy * barrel_length,
                4.0,
                turret_color,
            );
        }
        
        TileVisual::TransitionFade { progress } => {
            // Draw fade effect
            let alpha = (255.0 * (1.0 - progress)) as u8;
            draw_rectangle(x, y, size, size, Color::from_rgba(0, 0, 0, alpha));
        }
    }
}

/// Render a grid of tiles with visibility
pub fn render_tile_grid(
    tiles: &[(i32, i32, TileVisual)], // (x, y, visual)
    visible_tiles: &[(i32, i32)], // List of visible tile positions
    camera_x: f32,
    camera_y: f32,
) {
    // Create visibility set for fast lookup
    let visible_set: std::collections::HashSet<(i32, i32)> = visible_tiles.iter().cloned().collect();
    
    for (tile_x, tile_y, visual) in tiles {
        let world_x = *tile_x as f32 * TILE_SIZE;
        let world_y = *tile_y as f32 * TILE_SIZE;
        let screen_x = world_x - camera_x;
        let screen_y = world_y - camera_y;
        
        // Skip tiles outside screen
        if screen_x < -TILE_SIZE || screen_x > screen_width() ||
           screen_y < -TILE_SIZE || screen_y > screen_height() {
            continue;
        }
        
        // Check visibility
        let is_visible = visible_set.contains(&(*tile_x, *tile_y));
        
        if is_visible {
            render_tile_visual(visual, screen_x, screen_y, TILE_SIZE);
        } else {
            // Render as dark/fog
            draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, Color::from_rgba(20, 20, 25, 255));
        }
    }
}

