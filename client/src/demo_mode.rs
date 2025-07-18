use macroquad::prelude::*;
use shared::{TileVisual, Material, Direction, StationType, TILE_SIZE};
use crate::rendering::hybrid_tiles;
use std::collections::HashSet;

pub struct DemoMode {
    tiles: Vec<(i32, i32, TileVisual)>,
    visible_tiles: HashSet<(i32, i32)>,
    camera_x: f32,
    camera_y: f32,
    show_vision: bool,
}

impl DemoMode {
    pub fn new() -> Self {
        let mut demo = Self {
            tiles: Vec::new(),
            visible_tiles: HashSet::new(),
            camera_x: -200.0,
            camera_y: -200.0,
            show_vision: true,
        };
        
        // Create a demo mech interior
        demo.create_mech_interior();
        
        // Set initial visibility
        demo.update_visibility((10, 10));
        
        demo
    }
    
    fn create_mech_interior(&mut self) {
        // Ground floor (10x10)
        for y in 0..10 {
            for x in 0..10 {
                let is_wall = x == 0 || x == 9 || y == 0 || y == 9;
                let is_door = (x == 4 || x == 5) && y == 9;
                
                if is_door {
                    // Door area - just floor
                    self.tiles.push((x, y, TileVisual::Floor {
                        material: Material::Metal,
                        wear: 20,
                    }));
                } else if is_wall {
                    self.tiles.push((x, y, TileVisual::Wall {
                        material: Material::Metal,
                    }));
                } else {
                    self.tiles.push((x, y, TileVisual::Floor {
                        material: Material::Metal,
                        wear: 0,
                    }));
                }
            }
        }
        
        // Add resource drop-off area on roof
        self.tiles.push((4, -1, TileVisual::Floor {
            material: Material::Reinforced,
            wear: 0,
        }));
        self.tiles.push((5, -1, TileVisual::Floor {
            material: Material::Reinforced,
            wear: 0,
        }));
        
        // Add stations on ground floor
        self.tiles.push((2, 5, TileVisual::Station {
            station_type: StationType::Engine,
            active: true,
        }));
        self.tiles.push((7, 5, TileVisual::Station {
            station_type: StationType::Upgrade,
            active: false,
        }));
        
        // Upper floor (with windows)
        let floor2_offset = 12;
        for y in 0..10 {
            for x in 0..10 {
                let is_wall = x == 0 || x == 9 || y == 0 || y == 9;
                let is_window = is_wall && ((x == 5 && y == 0) || 
                                            (x == 0 && y == 5) || 
                                            (x == 9 && y == 5));
                
                if is_window {
                    let facing = if x == 0 {
                        Direction::Left
                    } else if x == 9 {
                        Direction::Right
                    } else {
                        Direction::Up
                    };
                    
                    self.tiles.push((x + floor2_offset, y, TileVisual::Window {
                        broken: false,
                        facing,
                    }));
                } else if is_wall {
                    self.tiles.push((x + floor2_offset, y, TileVisual::Wall {
                        material: Material::Reinforced,
                    }));
                } else {
                    self.tiles.push((x + floor2_offset, y, TileVisual::Floor {
                        material: Material::Metal,
                        wear: 0,
                    }));
                }
            }
        }
        
        // Add stations on upper floor
        self.tiles.push((floor2_offset + 2, 2, TileVisual::Station {
            station_type: StationType::WeaponLaser,
            active: true,
        }));
        self.tiles.push((floor2_offset + 7, 2, TileVisual::Station {
            station_type: StationType::Shield,
            active: false,
        }));
        self.tiles.push((floor2_offset + 5, 7, TileVisual::Station {
            station_type: StationType::Pilot,
            active: true,
        }));
        
        // Add a turret
        self.tiles.push((floor2_offset + 5, 2, TileVisual::Turret {
            facing: Direction::Up,
            firing: false,
        }));
    }
    
    fn update_visibility(&mut self, center: (i32, i32)) {
        self.visible_tiles.clear();
        
        if !self.show_vision {
            // Make everything visible
            for (x, y, _) in &self.tiles {
                self.visible_tiles.insert((*x, *y));
            }
            return;
        }
        
        // Simple radius-based visibility for demo
        let radius = 5;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= radius * radius {
                    self.visible_tiles.insert((center.0 + dx, center.1 + dy));
                }
            }
        }
        
        // Add window vision cones
        for (x, y, tile) in &self.tiles {
            if let TileVisual::Window { facing, broken: false } = tile {
                if self.visible_tiles.contains(&(*x, *y)) {
                    // Add extended vision through window
                    let (dx, dy) = match facing {
                        Direction::Up => (0, -1),
                        Direction::Down => (0, 1),
                        Direction::Left => (-1, 0),
                        Direction::Right => (1, 0),
                    };
                    
                    for i in 1..=8 {
                        self.visible_tiles.insert((x + dx * i, y + dy * i));
                        // Add some width to the cone
                        if i > 2 {
                            let perp_dx = if dx == 0 { 1 } else { 0 };
                            let perp_dy = if dy == 0 { 1 } else { 0 };
                            self.visible_tiles.insert((x + dx * i + perp_dx, y + dy * i + perp_dy));
                            self.visible_tiles.insert((x + dx * i - perp_dx, y + dy * i - perp_dy));
                        }
                    }
                }
            }
        }
    }
    
    pub async fn run(&mut self) {
        loop {
            clear_background(Color::from_rgba(10, 10, 15, 255));
            
            // Handle input
            if is_key_pressed(KeyCode::Escape) {
                break;
            }
            
            // Camera controls
            let speed = 200.0 * get_frame_time();
            if is_key_down(KeyCode::Left) {
                self.camera_x += speed;
            }
            if is_key_down(KeyCode::Right) {
                self.camera_x -= speed;
            }
            if is_key_down(KeyCode::Up) {
                self.camera_y += speed;
            }
            if is_key_down(KeyCode::Down) {
                self.camera_y -= speed;
            }
            
            // Toggle vision
            if is_key_pressed(KeyCode::V) {
                self.show_vision = !self.show_vision;
                let player_tile = (
                    ((-self.camera_x + screen_width() / 2.0) / TILE_SIZE) as i32,
                    ((-self.camera_y + screen_height() / 2.0) / TILE_SIZE) as i32,
                );
                self.update_visibility(player_tile);
            }
            
            // Update player position for visibility
            if self.show_vision {
                let player_tile = (
                    ((-self.camera_x + screen_width() / 2.0) / TILE_SIZE) as i32,
                    ((-self.camera_y + screen_height() / 2.0) / TILE_SIZE) as i32,
                );
                self.update_visibility(player_tile);
            }
            
            // Render tiles
            let visible_vec: Vec<(i32, i32)> = self.visible_tiles.iter().cloned().collect();
            hybrid_tiles::render_tile_grid(&self.tiles, &visible_vec, self.camera_x, self.camera_y);
            
            // Draw player indicator
            let player_x = screen_width() / 2.0;
            let player_y = screen_height() / 2.0;
            draw_circle(player_x, player_y, 8.0, YELLOW);
            
            // UI
            draw_text("Hybrid Tile System Demo", 10.0, 30.0, 24.0, WHITE);
            draw_text("Arrow keys: Move camera", 10.0, 55.0, 18.0, LIGHTGRAY);
            draw_text("V: Toggle vision/fog of war", 10.0, 75.0, 18.0, LIGHTGRAY);
            draw_text("ESC: Exit demo", 10.0, 95.0, 18.0, LIGHTGRAY);
            
            let vision_text = if self.show_vision {
                "Vision: ON (with fog of war)"
            } else {
                "Vision: OFF (all visible)"
            };
            draw_text(vision_text, 10.0, 120.0, 18.0, GREEN);
            
            // Show what's at player position
            let player_tile = (
                ((-self.camera_x + screen_width() / 2.0) / TILE_SIZE) as i32,
                ((-self.camera_y + screen_height() / 2.0) / TILE_SIZE) as i32,
            );
            
            if let Some((_, _, tile)) = self.tiles.iter().find(|(x, y, _)| *x == player_tile.0 && *y == player_tile.1) {
                let tile_desc = match tile {
                    TileVisual::Floor { material, wear } => format!("Floor ({:?}, wear: {})", material, wear),
                    TileVisual::Wall { material } => format!("Wall ({:?})", material),
                    TileVisual::Window { facing, broken } => format!("Window ({:?}, broken: {})", facing, broken),
                    TileVisual::Station { station_type, active } => format!("Station ({:?}, active: {})", station_type, active),
                    TileVisual::Turret { facing, firing } => format!("Turret ({:?}, firing: {})", facing, firing),
                    TileVisual::TransitionFade { progress } => format!("Transition ({}%)", (progress * 100.0) as i32),
                };
                draw_text(&format!("Current tile: {}", tile_desc), 10.0, screen_height() - 20.0, 18.0, WHITE);
            }
            
            next_frame().await;
        }
    }
}