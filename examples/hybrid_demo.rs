use shared::{
    components::*, constants::*, tile_entity::*, tile_migration::*, vision::*, Color, Direction,
    ResourceType, StationType, TilePos, WorldPos,
};
use std::collections::HashMap;
use uuid::Uuid;

fn main() {
    println!("=== Hybrid Tile-Entity System Demo ===\n");

    // Create a tile map
    let mut tile_map = TileMap::new();

    // Create a simple room layout
    println!("Creating room layout...");
    create_room(&mut tile_map, TilePos::new(5, 5), 10, 8);

    // Create entity storage
    let mut entities = MockEntityStorage::new();

    // Add some entities
    println!("\nAdding station entities...");
    let station1 =
        create_station_entity(&mut entities, TilePos::new(7, 7), StationType::WeaponLaser);
    let station2 = create_station_entity(&mut entities, TilePos::new(12, 7), StationType::Shield);

    // Add entities to tile map
    tile_map.set_entity_tile(TilePos::new(7, 7), station1);
    tile_map.set_entity_tile(TilePos::new(12, 7), station2);

    // Test vision system
    println!("\nTesting vision system...");
    let mut vision_system = VisionSystem::new();
    let viewer_id = Uuid::new_v4();
    let viewer_pos = WorldPos::new(10.0 * TILE_SIZE, 10.0 * TILE_SIZE);

    let visibility = vision_system.calculate_visibility(
        viewer_id,
        viewer_pos,
        10.0 * TILE_SIZE,
        &tile_map,
        &entities,
    );

    println!(
        "Visible tiles from ({}, {}): {} tiles",
        viewer_pos.x,
        viewer_pos.y,
        visibility.visible_tiles.len()
    );

    // Test movement
    println!("\nTesting movement system...");
    let player_id = Uuid::new_v4();
    let target_pos = WorldPos::new(8.0 * TILE_SIZE, 8.0 * TILE_SIZE);

    match handle_movement(&tile_map, &entities, player_id, target_pos) {
        Ok(()) => println!("Movement to ({}, {}) is valid", target_pos.x, target_pos.y),
        Err(e) => println!("Movement blocked: {:?}", e),
    }

    // Test window vision
    println!("\nTesting window vision...");
    let window_vision = WindowVision::new(5.0 * TILE_SIZE, 10.0 * TILE_SIZE);
    let window_result = window_vision.calculate_window_visibility(
        viewer_pos, true, // inside mech
        &tile_map, &entities,
    );

    println!(
        "Found {} windows providing extended vision",
        window_result.window_views.len()
    );

    // Demonstrate tile content inspection
    println!("\nInspecting tile contents...");
    inspect_area(&tile_map, &entities, TilePos::new(5, 5), 5);

    println!("\n=== Demo Complete ===");
}

fn create_room(tile_map: &mut TileMap, corner: TilePos, width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let pos = TilePos::new(corner.x + x as i32, corner.y + y as i32);

            // Walls on edges
            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                if x == width / 2 && y == 0 {
                    // Window on north wall
                    tile_map.set_static_tile(
                        pos,
                        StaticTile::Window {
                            facing: Direction::Up,
                        },
                    );
                } else {
                    tile_map.set_static_tile(pos, StaticTile::MetalWall);
                }
            } else {
                // Floor inside
                tile_map.set_static_tile(pos, StaticTile::MetalFloor);
            }
        }
    }
}

fn create_station_entity(
    entities: &mut MockEntityStorage,
    pos: TilePos,
    station_type: StationType,
) -> Uuid {
    let entity_id = Uuid::new_v4();
    let position = Position {
        tile: pos,
        world: pos.to_world(),
        floor: Some(0),
        mech_id: None,
    };

    entities.add_position(entity_id, position);
    entities.add_station(
        entity_id,
        Station {
            station_type,
            interaction_range: 1.5,
            power_required: 50.0,
            operating: false,
        },
    );

    println!("Created {:?} station at {:?}", station_type, pos);
    entity_id
}

fn inspect_area(tile_map: &TileMap, entities: &MockEntityStorage, center: TilePos, radius: i32) {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let pos = TilePos::new(center.x + dx, center.y + dy);
            let world_pos = pos.to_world();

            if let Some(content) = tile_map.get_tile_at(world_pos) {
                match content {
                    TileContent::Static(tile) => {
                        print!("{}", tile_to_char(tile));
                    }
                    TileContent::Entity(id) => {
                        if let Some(station) = entities.get_station(id) {
                            print!("S");
                        } else {
                            print!("E");
                        }
                    }
                    TileContent::Empty => print!("."),
                }
            } else {
                print!(" ");
            }
        }
        println!();
    }
}

fn tile_to_char(tile: StaticTile) -> char {
    match tile {
        StaticTile::MetalFloor => '.',
        StaticTile::MetalWall => '#',
        StaticTile::Window { .. } => 'W',
        StaticTile::TransitionZone { .. } => 'T',
        _ => '?',
    }
}

// Mock entity storage for demo
struct MockEntityStorage {
    positions: HashMap<Uuid, Position>,
    stations: HashMap<Uuid, Station>,
}

impl MockEntityStorage {
    fn new() -> Self {
        Self {
            positions: HashMap::new(),
            stations: HashMap::new(),
        }
    }

    fn add_position(&mut self, id: Uuid, pos: Position) {
        self.positions.insert(id, pos);
    }

    fn add_station(&mut self, id: Uuid, station: Station) {
        self.stations.insert(id, station);
    }
}

impl ComponentStorage for MockEntityStorage {
    fn get_position(&self, entity: Uuid) -> Option<&Position> {
        self.positions.get(&entity)
    }

    fn get_station(&self, entity: Uuid) -> Option<&Station> {
        self.stations.get(&entity)
    }

    fn get_renderable(&self, _entity: Uuid) -> Option<&Renderable> {
        None
    }

    fn get_solid(&self, _entity: Uuid) -> Option<&Solid> {
        None
    }

    fn get_opaque(&self, _entity: Uuid) -> Option<&Opaque> {
        None
    }

    fn get_position_mut(&mut self, entity: Uuid) -> Option<&mut Position> {
        self.positions.get_mut(&entity)
    }

    fn get_station_mut(&mut self, entity: Uuid) -> Option<&mut Station> {
        self.stations.get_mut(&entity)
    }
}
