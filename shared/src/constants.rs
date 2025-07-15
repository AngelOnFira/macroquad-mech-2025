// Game dimensions
pub const TILE_SIZE: f32 = 32.0;
pub const ARENA_WIDTH_TILES: i32 = 100;
pub const ARENA_HEIGHT_TILES: i32 = 100;
pub const MECH_SIZE_TILES: i32 = 10;

// Player settings
pub const MAX_DISTANCE_FROM_MECH: f32 = 15.0; // tiles

// Mech internals
pub const MECH_FLOORS: usize = 3;
pub const FLOOR_HEIGHT_TILES: i32 = 6;
pub const FLOOR_WIDTH_TILES: i32 = 20;

// Resources
pub const RESOURCE_TYPES: usize = 4; // Scrap Metal, Computer Components, Wiring, Batteries

// Network
pub const SERVER_PORT: u16 = 14191;