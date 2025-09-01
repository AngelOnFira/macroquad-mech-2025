use macroquad::prelude::*;
use shared::{Direction, Material, StationType, TileVisual, TILE_SIZE};
use std::collections::{HashMap, HashSet};

// Arrow drawing utility (simplified version of ArrowRenderer)
fn draw_directional_arrow(
    center_x: f32,
    center_y: f32,
    size: f32,
    direction: Direction,
    color: Color,
) {
    let arrow_size = size * 0.3;

    match direction {
        Direction::Up => {
            draw_triangle(
                Vec2::new(center_x, center_y - arrow_size),
                Vec2::new(center_x - arrow_size / 2.0, center_y),
                Vec2::new(center_x + arrow_size / 2.0, center_y),
                color,
            );
        }
        Direction::Down => {
            draw_triangle(
                Vec2::new(center_x, center_y + arrow_size),
                Vec2::new(center_x - arrow_size / 2.0, center_y),
                Vec2::new(center_x + arrow_size / 2.0, center_y),
                color,
            );
        }
        Direction::Left => {
            draw_triangle(
                Vec2::new(center_x - arrow_size, center_y),
                Vec2::new(center_x, center_y - arrow_size / 2.0),
                Vec2::new(center_x, center_y + arrow_size / 2.0),
                color,
            );
        }
        Direction::Right => {
            draw_triangle(
                Vec2::new(center_x + arrow_size, center_y),
                Vec2::new(center_x, center_y - arrow_size / 2.0),
                Vec2::new(center_x, center_y + arrow_size / 2.0),
                color,
            );
        }
    }
}

// Simple raycasting for line of sight
fn cast_ray(
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
    max_dist: f32,
    check_blocking: impl Fn(i32, i32) -> bool,
) -> bool {
    let dx = to_x - from_x;
    let dy = to_y - from_y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist > max_dist {
        return false;
    }

    // Number of steps for raycasting
    let steps = (dist / (TILE_SIZE * 0.25)).ceil() as i32;

    for i in 1..steps {
        let t = i as f32 / steps as f32;
        let check_x = from_x + dx * t;
        let check_y = from_y + dy * t;
        let tile_x = (check_x / TILE_SIZE).floor() as i32;
        let tile_y = (check_y / TILE_SIZE).floor() as i32;

        if check_blocking(tile_x, tile_y) {
            return false;
        }
    }

    true
}

#[derive(Clone, Copy, PartialEq)]
enum LayerType {
    Ground,        // The world outside
    MechFloor(u8), // Floor 0, 1, 2 of a mech
}

struct Tile {
    visual: TileVisual,
    blocks_vision: bool,
    walkable: bool,
}

struct DemoMode {
    // World tiles (ground layer)
    ground_tiles: HashMap<(i32, i32), Tile>,

    // Mech interior tiles by floor
    mech_tiles: HashMap<(u8, i32, i32), Tile>, // (floor, x, y)

    // Player state
    player_x: f32,
    player_y: f32,
    player_velocity: (f32, f32),
    current_layer: LayerType,

    // Camera
    camera_x: f32,
    camera_y: f32,

    // Visibility
    visible_tiles: HashSet<(i32, i32)>,

    // Transition state
    transition_progress: f32,
    transition_from: Option<LayerType>,
    transition_to: Option<LayerType>,

    // Mech properties
    mech_pos: (i32, i32), // Position of mech in world
}

impl DemoMode {
    fn new() -> Self {
        let mut demo = Self {
            ground_tiles: HashMap::new(),
            mech_tiles: HashMap::new(),
            player_x: 5.5 * TILE_SIZE,
            player_y: 8.5 * TILE_SIZE,
            player_velocity: (0.0, 0.0),
            current_layer: LayerType::Ground,
            camera_x: 0.0,
            camera_y: 0.0,
            visible_tiles: HashSet::new(),
            transition_progress: 0.0,
            transition_from: None,
            transition_to: None,
            mech_pos: (10, 10),
        };

        demo.create_world();
        demo.update_visibility();

        demo
    }

    fn create_world(&mut self) {
        // Create ground layer - grass everywhere with some rocks
        for x in -20..40 {
            for y in -20..40 {
                let tile = if rand::gen_range(0, 100) < 5 {
                    // Occasional rock
                    Tile {
                        visual: TileVisual::Wall {
                            material: Material::Damaged,
                        },
                        blocks_vision: true,
                        walkable: false,
                    }
                } else {
                    // Grass
                    Tile {
                        visual: TileVisual::Floor {
                            material: Material::Damaged,
                            wear: 50,
                        },
                        blocks_vision: false,
                        walkable: true,
                    }
                };
                self.ground_tiles.insert((x, y), tile);
            }
        }

        // Place mech exterior on ground (10x10 footprint at mech_pos)
        for dx in 0..10 {
            for dy in 0..10 {
                let x = self.mech_pos.0 + dx;
                let y = self.mech_pos.1 + dy;

                // Mech exterior is metal floor
                let is_entrance = dx >= 4 && dx <= 5 && dy == 9;

                let tile = Tile {
                    visual: TileVisual::Floor {
                        material: if is_entrance {
                            Material::Reinforced
                        } else {
                            Material::Metal
                        },
                        wear: if is_entrance { 200 } else { 0 }, // Visual indicator for entrance
                    },
                    blocks_vision: false,
                    walkable: true,
                };
                self.ground_tiles.insert((x, y), tile);
            }
        }

        // Create mech interior floors (3 floors)
        for floor in 0..3 {
            self.create_mech_floor(floor);
        }
    }

    fn create_mech_floor(&mut self, floor: u8) {
        // Each floor is 10x10
        for x in 0..10 {
            for y in 0..10 {
                let is_wall = x == 0 || x == 9 || y == 0 || y == 9;
                let is_entrance = floor == 0 && x >= 4 && x <= 5 && y == 9;
                let is_stairs = (x == 2 && y == 2) || (x == 7 && y == 7);

                let tile = if is_entrance {
                    // Entrance zone
                    Tile {
                        visual: TileVisual::Floor {
                            material: Material::Reinforced,
                            wear: 200,
                        },
                        blocks_vision: false,
                        walkable: true,
                    }
                } else if is_stairs && floor < 2 {
                    // Stairs up
                    Tile {
                        visual: TileVisual::Floor {
                            material: Material::Reinforced,
                            wear: 150,
                        },
                        blocks_vision: false,
                        walkable: true,
                    }
                } else if is_wall {
                    // Check for windows on upper floors
                    let is_window = floor > 0
                        && ((x == 5 && y == 0) || (x == 0 && y == 5) || (x == 9 && y == 5));

                    if is_window {
                        let facing = if x == 0 {
                            Direction::Left
                        } else if x == 9 {
                            Direction::Right
                        } else {
                            Direction::Up
                        };

                        Tile {
                            visual: TileVisual::Window {
                                broken: false,
                                facing,
                            },
                            blocks_vision: false, // Windows don't block vision
                            walkable: false,
                        }
                    } else {
                        Tile {
                            visual: TileVisual::Wall {
                                material: if floor > 0 {
                                    Material::Reinforced
                                } else {
                                    Material::Metal
                                },
                            },
                            blocks_vision: true,
                            walkable: false,
                        }
                    }
                } else {
                    // Floor tile - add stations
                    let station = match floor {
                        0 => {
                            if x == 2 && y == 5 {
                                Some(StationType::Engine)
                            } else if x == 7 && y == 5 {
                                Some(StationType::Electrical)
                            } else if x == 5 && y == 2 {
                                Some(StationType::Upgrade)
                            } else {
                                None
                            }
                        }
                        1 => {
                            if x == 2 && y == 5 {
                                Some(StationType::WeaponLaser)
                            } else if x == 7 && y == 5 {
                                Some(StationType::WeaponProjectile)
                            } else if x == 5 && y == 2 {
                                None // Turret position
                            } else {
                                None
                            }
                        }
                        2 => {
                            if x == 5 && y == 5 {
                                Some(StationType::Pilot)
                            } else if x == 2 && y == 2 {
                                Some(StationType::Shield)
                            } else if x == 7 && y == 7 {
                                Some(StationType::Repair)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    if let Some(station_type) = station {
                        Tile {
                            visual: TileVisual::Station {
                                station_type,
                                active: rand::gen_range(0, 2) == 0,
                            },
                            blocks_vision: false,
                            walkable: true,
                        }
                    } else if floor == 1 && x == 5 && y == 2 {
                        // Turret
                        Tile {
                            visual: TileVisual::Turret {
                                facing: Direction::Up,
                                firing: false,
                            },
                            blocks_vision: true,
                            walkable: false,
                        }
                    } else {
                        Tile {
                            visual: TileVisual::Floor {
                                material: Material::Metal,
                                wear: 0,
                            },
                            blocks_vision: false,
                            walkable: true,
                        }
                    }
                };

                self.mech_tiles.insert((floor, x, y), tile);
            }
        }
    }

    fn get_player_tile_pos(&self) -> (i32, i32) {
        (
            (self.player_x / TILE_SIZE).floor() as i32,
            (self.player_y / TILE_SIZE).floor() as i32,
        )
    }

    fn update_visibility(&mut self) {
        self.visible_tiles.clear();

        let (player_tile_x, player_tile_y) = self.get_player_tile_pos();
        let player_center_x = self.player_x;
        let player_center_y = self.player_y;

        match self.current_layer {
            LayerType::Ground => {
                // On ground - use simple radius for now
                let radius = 8;
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let tile_x = player_tile_x + dx;
                        let tile_y = player_tile_y + dy;
                        let tile_center_x = (tile_x as f32 + 0.5) * TILE_SIZE;
                        let tile_center_y = (tile_y as f32 + 0.5) * TILE_SIZE;

                        // Check line of sight
                        let has_los = cast_ray(
                            player_center_x,
                            player_center_y,
                            tile_center_x,
                            tile_center_y,
                            radius as f32 * TILE_SIZE,
                            |x, y| {
                                if let Some(tile) = self.ground_tiles.get(&(x, y)) {
                                    tile.blocks_vision
                                } else {
                                    false
                                }
                            },
                        );

                        if has_los {
                            self.visible_tiles.insert((tile_x, tile_y));
                        }
                    }
                }
            }
            LayerType::MechFloor(floor) => {
                // Inside mech - everything is dark except what we can see
                let radius = 6;

                // First, check what we can see inside the mech
                for y in 0..10 {
                    for x in 0..10 {
                        let tile_world_x = self.mech_pos.0 + x;
                        let tile_world_y = self.mech_pos.1 + y;
                        let tile_center_x = (tile_world_x as f32 + 0.5) * TILE_SIZE;
                        let tile_center_y = (tile_world_y as f32 + 0.5) * TILE_SIZE;

                        // Check line of sight within mech
                        let has_los = cast_ray(
                            player_center_x,
                            player_center_y,
                            tile_center_x,
                            tile_center_y,
                            radius as f32 * TILE_SIZE,
                            |check_x, check_y| {
                                // Convert world coords to mech-relative
                                let rel_x = check_x - self.mech_pos.0;
                                let rel_y = check_y - self.mech_pos.1;

                                if rel_x >= 0 && rel_x < 10 && rel_y >= 0 && rel_y < 10 {
                                    if let Some(tile) = self.mech_tiles.get(&(floor, rel_x, rel_y))
                                    {
                                        tile.blocks_vision
                                    } else {
                                        false
                                    }
                                } else {
                                    true // Outside mech blocks vision
                                }
                            },
                        );

                        if has_los {
                            self.visible_tiles.insert((tile_world_x, tile_world_y));

                            // If this is a window, cast vision through it
                            if let Some(tile) = self.mech_tiles.get(&(floor, x, y)) {
                                if let TileVisual::Window {
                                    facing,
                                    broken: false,
                                } = tile.visual
                                {
                                    // Can we see the window?
                                    if has_los {
                                        // Cast vision cone through window
                                        self.cast_window_vision(tile_world_x, tile_world_y, facing);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn cast_window_vision(&mut self, window_x: i32, window_y: i32, facing: Direction) {
        let (dx, dy) = match facing {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };

        // Cast vision cone from window
        let window_center_x = (window_x as f32 + 0.5) * TILE_SIZE;
        let window_center_y = (window_y as f32 + 0.5) * TILE_SIZE;

        // Check tiles in cone
        for dist in 1..=15 {
            for width in -(dist / 3)..=(dist / 3) {
                let check_x = window_x + dx * dist + dy * width;
                let check_y = window_y + dy * dist - dx * width;

                let tile_center_x = (check_x as f32 + 0.5) * TILE_SIZE;
                let tile_center_y = (check_y as f32 + 0.5) * TILE_SIZE;

                // Check line of sight from window to tile
                let has_los = cast_ray(
                    window_center_x,
                    window_center_y,
                    tile_center_x,
                    tile_center_y,
                    15.0 * TILE_SIZE,
                    |x, y| {
                        if let Some(tile) = self.ground_tiles.get(&(x, y)) {
                            tile.blocks_vision
                        } else {
                            false
                        }
                    },
                );

                if has_los {
                    self.visible_tiles.insert((check_x, check_y));
                }
            }
        }
    }

    fn is_walkable(&self, world_x: i32, world_y: i32) -> bool {
        match self.current_layer {
            LayerType::Ground => self
                .ground_tiles
                .get(&(world_x, world_y))
                .map(|t| t.walkable)
                .unwrap_or(false),
            LayerType::MechFloor(floor) => {
                let rel_x = world_x - self.mech_pos.0;
                let rel_y = world_y - self.mech_pos.1;

                if rel_x >= 0 && rel_x < 10 && rel_y >= 0 && rel_y < 10 {
                    self.mech_tiles
                        .get(&(floor, rel_x, rel_y))
                        .map(|t| t.walkable)
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }

    fn check_transitions(&mut self) {
        let (tile_x, tile_y) = self.get_player_tile_pos();

        match self.current_layer {
            LayerType::Ground => {
                // Check if on mech entrance
                let rel_x = tile_x - self.mech_pos.0;
                let rel_y = tile_y - self.mech_pos.1;

                if rel_x >= 4 && rel_x <= 5 && rel_y == 9 {
                    // On entrance - start transition
                    self.transition_from = Some(self.current_layer);
                    self.transition_to = Some(LayerType::MechFloor(0));
                    self.transition_progress = 0.0;
                }
            }
            LayerType::MechFloor(floor) => {
                let rel_x = tile_x - self.mech_pos.0;
                let rel_y = tile_y - self.mech_pos.1;

                // Check for entrance (exit)
                if floor == 0 && rel_x >= 4 && rel_x <= 5 && rel_y == 9 {
                    self.transition_from = Some(self.current_layer);
                    self.transition_to = Some(LayerType::Ground);
                    self.transition_progress = 0.0;
                }

                // Check for stairs
                if (rel_x == 2 && rel_y == 2) || (rel_x == 7 && rel_y == 7) {
                    if floor < 2 {
                        // Can go up
                        self.transition_from = Some(self.current_layer);
                        self.transition_to = Some(LayerType::MechFloor(floor + 1));
                        self.transition_progress = 0.0;
                    }
                    if floor > 0 {
                        // Can also go down from same position
                        // For demo, we'll use Q/E keys to choose direction
                    }
                }
            }
        }
    }

    fn render_tile(&self, tile: &Tile, screen_x: f32, screen_y: f32, alpha: f32) {
        let mut color = match &tile.visual {
            TileVisual::Floor { material, wear } => {
                let base_color = match material {
                    Material::Metal => Color::from_rgba(100, 100, 110, 255),
                    Material::Reinforced => Color::from_rgba(80, 80, 90, 255),
                    Material::Damaged => Color::from_rgba(60, 50, 50, 255),
                };

                let wear_factor = 1.0 - (*wear as f32 / 255.0) * 0.3;
                Color::new(
                    base_color.r * wear_factor,
                    base_color.g * wear_factor,
                    base_color.b * wear_factor,
                    base_color.a,
                )
            }
            _ => WHITE,
        };

        // Apply alpha for transitions
        color.a *= alpha;

        match &tile.visual {
            TileVisual::Floor { .. } => {
                draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, color);
                let line_color = Color::new(0.235, 0.235, 0.275, color.a * 0.4);
                draw_rectangle_lines(screen_x, screen_y, TILE_SIZE, TILE_SIZE, 1.0, line_color);
            }
            _ => {
                // Use the standard rendering but with alpha
                self.render_tile_visual_with_alpha(
                    &tile.visual,
                    screen_x,
                    screen_y,
                    TILE_SIZE,
                    alpha,
                );
            }
        }
    }

    fn render_tile_visual_with_alpha(
        &self,
        visual: &TileVisual,
        x: f32,
        y: f32,
        size: f32,
        alpha: f32,
    ) {
        // Similar to the original render_tile_visual but with alpha support
        match visual {
            TileVisual::Wall { material } => {
                let mut color = match material {
                    Material::Metal => Color::from_rgba(50, 50, 60, 255),
                    Material::Reinforced => Color::from_rgba(30, 30, 40, 255),
                    Material::Damaged => Color::from_rgba(40, 30, 30, 255),
                };
                color.a *= alpha;

                draw_rectangle(x, y, size, size, color);
                let mut highlight = Color::from_rgba(80, 80, 90, 255);
                highlight.a *= alpha;
                draw_line(x, y, x + size, y, 2.0, highlight);
            }
            TileVisual::Window { broken, facing } => {
                let mut frame_color = Color::from_rgba(40, 40, 50, 255);
                frame_color.a *= alpha;
                draw_rectangle(x, y, size, size, frame_color);

                if !broken {
                    let mut glass_color = Color::from_rgba(100, 120, 140, 100);
                    glass_color.a *= alpha;
                    draw_rectangle(x + 2.0, y + 2.0, size - 4.0, size - 4.0, glass_color);

                    // Draw direction arrow using utility function
                    let center_x = x + size / 2.0;
                    let center_y = y + size / 2.0;
                    let mut arrow_color = Color::from_rgba(200, 200, 210, 150);
                    arrow_color.a *= alpha;

                    draw_directional_arrow(center_x, center_y, size, *facing, arrow_color);
                }
            }
            TileVisual::Station {
                station_type,
                active,
            } => {
                let mut floor_color = Color::from_rgba(100, 100, 110, 255);
                floor_color.a *= alpha;
                draw_rectangle(x, y, size, size, floor_color);

                let mut station_color = if *active {
                    Color::from_rgba(100, 255, 100, 255)
                } else {
                    Color::from_rgba(150, 150, 160, 255)
                };
                station_color.a *= alpha;

                let station_size = size * 0.8;
                let offset = size * 0.1;
                draw_rectangle(
                    x + offset,
                    y + offset,
                    station_size,
                    station_size,
                    station_color,
                );

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
                let mut base_color = Color::from_rgba(60, 60, 70, 255);
                base_color.a *= alpha;
                draw_rectangle(x, y, size, size, base_color);

                let mut turret_color = if *firing {
                    Color::from_rgba(255, 100, 100, 255)
                } else {
                    Color::from_rgba(120, 120, 130, 255)
                };
                turret_color.a *= alpha;

                let center_x = x + size / 2.0;
                let center_y = y + size / 2.0;
                let turret_radius = size * 0.3;

                draw_circle(center_x, center_y, turret_radius, turret_color);

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
            _ => {}
        }
    }

    async fn run(&mut self) {
        loop {
            clear_background(Color::from_rgba(10, 10, 15, 255));

            if is_key_pressed(KeyCode::Escape) {
                break;
            }

            // Handle continuous movement
            let speed = 150.0; // pixels per second
            let dt = get_frame_time();

            // Update velocity based on input
            self.player_velocity = (0.0, 0.0);

            if is_key_down(KeyCode::W) {
                self.player_velocity.1 = -speed;
            }
            if is_key_down(KeyCode::S) {
                self.player_velocity.1 = speed;
            }
            if is_key_down(KeyCode::A) {
                self.player_velocity.0 = -speed;
            }
            if is_key_down(KeyCode::D) {
                self.player_velocity.0 = speed;
            }

            // Normalize diagonal movement
            let vel_magnitude = (self.player_velocity.0 * self.player_velocity.0
                + self.player_velocity.1 * self.player_velocity.1)
                .sqrt();
            if vel_magnitude > 0.0 {
                self.player_velocity.0 = self.player_velocity.0 / vel_magnitude * speed;
                self.player_velocity.1 = self.player_velocity.1 / vel_magnitude * speed;
            }

            // Try to move
            let new_x = self.player_x + self.player_velocity.0 * dt;
            let new_y = self.player_y + self.player_velocity.1 * dt;

            // Check collision for new position
            let new_tile_x = (new_x / TILE_SIZE).floor() as i32;
            let new_tile_y = (new_y / TILE_SIZE).floor() as i32;

            if self.is_walkable(new_tile_x, new_tile_y) {
                self.player_x = new_x;
                self.player_y = new_y;

                // Check for transitions
                if self.transition_from.is_none() {
                    self.check_transitions();
                }
            }

            // Update transitions
            if self.transition_from.is_some() {
                self.transition_progress += dt * 2.0; // 0.5 second transition

                if self.transition_progress >= 1.0 {
                    // Complete transition
                    if let Some(new_layer) = self.transition_to {
                        self.current_layer = new_layer;
                    }
                    self.transition_from = None;
                    self.transition_to = None;
                    self.transition_progress = 0.0;
                }
            }

            // Handle stair navigation
            if let LayerType::MechFloor(floor) = self.current_layer {
                let (tile_x, tile_y) = self.get_player_tile_pos();
                let rel_x = tile_x - self.mech_pos.0;
                let rel_y = tile_y - self.mech_pos.1;

                if (rel_x == 2 && rel_y == 2) || (rel_x == 7 && rel_y == 7) {
                    if is_key_pressed(KeyCode::Q) && floor > 0 && self.transition_from.is_none() {
                        // Go up a floor
                        self.transition_from = Some(self.current_layer);
                        self.transition_to = Some(LayerType::MechFloor(floor - 1));
                        self.transition_progress = 0.0;
                    }
                    if is_key_pressed(KeyCode::E) && floor < 2 && self.transition_from.is_none() {
                        // Go down a floor
                        self.transition_from = Some(self.current_layer);
                        self.transition_to = Some(LayerType::MechFloor(floor + 1));
                        self.transition_progress = 0.0;
                    }
                }
            }

            // Update visibility
            self.update_visibility();

            // Update camera to follow player smoothly
            let target_cam_x = self.player_x - screen_width() / 2.0;
            let target_cam_y = self.player_y - screen_height() / 2.0;

            self.camera_x += (target_cam_x - self.camera_x) * 0.1;
            self.camera_y += (target_cam_y - self.camera_y) * 0.1;

            // Render based on current layer and transition state
            let layers_to_render = if let Some(from_layer) = self.transition_from {
                vec![
                    (from_layer, 1.0 - self.transition_progress),
                    (self.transition_to.unwrap(), self.transition_progress),
                ]
            } else {
                vec![(self.current_layer, 1.0)]
            };

            // Render each active layer
            for (layer, alpha) in layers_to_render {
                match layer {
                    LayerType::Ground => {
                        // Render ground tiles
                        for (pos, tile) in &self.ground_tiles {
                            let world_x = pos.0 as f32 * TILE_SIZE;
                            let world_y = pos.1 as f32 * TILE_SIZE;
                            let screen_x = world_x - self.camera_x;
                            let screen_y = world_y - self.camera_y;

                            if screen_x < -TILE_SIZE
                                || screen_x > screen_width()
                                || screen_y < -TILE_SIZE
                                || screen_y > screen_height()
                            {
                                continue;
                            }

                            if self.visible_tiles.contains(pos) {
                                self.render_tile(tile, screen_x, screen_y, alpha);
                            }
                        }
                    }
                    LayerType::MechFloor(floor) => {
                        // Render mech interior - everything is dark unless visible
                        for y in 0..10 {
                            for x in 0..10 {
                                let world_x = (self.mech_pos.0 + x) as f32 * TILE_SIZE;
                                let world_y = (self.mech_pos.1 + y) as f32 * TILE_SIZE;
                                let screen_x = world_x - self.camera_x;
                                let screen_y = world_y - self.camera_y;

                                if screen_x < -TILE_SIZE
                                    || screen_x > screen_width()
                                    || screen_y < -TILE_SIZE
                                    || screen_y > screen_height()
                                {
                                    continue;
                                }

                                let world_pos = (self.mech_pos.0 + x, self.mech_pos.1 + y);

                                if self.visible_tiles.contains(&world_pos) {
                                    // Render the tile
                                    if let Some(tile) = self.mech_tiles.get(&(floor, x, y)) {
                                        self.render_tile(tile, screen_x, screen_y, alpha);
                                    }
                                } else {
                                    // Draw darkness
                                    let mut dark = Color::from_rgba(0, 0, 0, 255);
                                    dark.a *= alpha;
                                    draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, dark);
                                }
                            }
                        }

                        // Also render visible ground tiles through windows
                        if alpha > 0.5 {
                            // Only when this is the primary layer
                            for (pos, tile) in &self.ground_tiles {
                                if self.visible_tiles.contains(pos) {
                                    let world_x = pos.0 as f32 * TILE_SIZE;
                                    let world_y = pos.1 as f32 * TILE_SIZE;
                                    let screen_x = world_x - self.camera_x;
                                    let screen_y = world_y - self.camera_y;

                                    if screen_x >= -TILE_SIZE
                                        && screen_x <= screen_width()
                                        && screen_y >= -TILE_SIZE
                                        && screen_y <= screen_height()
                                    {
                                        self.render_tile(tile, screen_x, screen_y, alpha * 0.8);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Draw player
            let player_screen_x = self.player_x - self.camera_x;
            let player_screen_y = self.player_y - self.camera_y;
            draw_circle(player_screen_x, player_screen_y, 8.0, YELLOW);

            // UI
            draw_text("Hybrid Tile System Demo - Layered", 10.0, 30.0, 24.0, WHITE);
            draw_text("WASD: Move (continuous)", 10.0, 55.0, 18.0, LIGHTGRAY);
            draw_text(
                "Walk onto entrance/stairs to transition",
                10.0,
                75.0,
                18.0,
                LIGHTGRAY,
            );
            draw_text(
                "Q/E: Go up/down when on stairs",
                10.0,
                95.0,
                18.0,
                LIGHTGRAY,
            );
            draw_text("ESC: Exit demo", 10.0, 115.0, 18.0, LIGHTGRAY);

            let layer_text = match self.current_layer {
                LayerType::Ground => "Ground Level".to_string(),
                LayerType::MechFloor(f) => format!("Mech Floor {}", f + 1),
            };
            draw_text(
                &format!("Current: {}", layer_text),
                10.0,
                140.0,
                18.0,
                GREEN,
            );

            // Show transition state
            if self.transition_from.is_some() {
                draw_text("Transitioning...", 10.0, 165.0, 18.0, YELLOW);
            }

            // Show if on stairs
            if let LayerType::MechFloor(floor) = self.current_layer {
                let (tile_x, tile_y) = self.get_player_tile_pos();
                let rel_x = tile_x - self.mech_pos.0;
                let rel_y = tile_y - self.mech_pos.1;

                if (rel_x == 2 && rel_y == 2) || (rel_x == 7 && rel_y == 7) {
                    let can_go_up = floor > 0;
                    let can_go_down = floor < 2;

                    if can_go_up && can_go_down {
                        draw_text("On stairs! Q: Up, E: Down", 10.0, 190.0, 18.0, YELLOW);
                    } else if can_go_up {
                        draw_text("On stairs! Q: Go up", 10.0, 190.0, 18.0, YELLOW);
                    } else if can_go_down {
                        draw_text("On stairs! E: Go down", 10.0, 190.0, 18.0, YELLOW);
                    }
                }
            }

            next_frame().await;
        }
    }
}

#[macroquad::main("Mech Battle Arena - Layered Demo")]
async fn main() {
    let mut demo = DemoMode::new();
    demo.run().await;
}
