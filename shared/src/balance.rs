// Game balance constants - all magic numbers extracted to one place

// ===== Spawning and Initial Setup =====
pub const RED_MECH_SPAWN: (i32, i32) = (20, 20);
pub const BLUE_MECH_SPAWN: (i32, i32) = (80, 80);
pub const RED_PLAYER_SPAWN: (f32, f32) = (15.0, 20.0);
pub const BLUE_PLAYER_SPAWN: (f32, f32) = (75.0, 80.0);

// ===== Mech Stats =====
pub const MECH_INITIAL_HEALTH: u32 = 100;
pub const MECH_MAX_HEALTH: u32 = 100;
pub const MECH_INITIAL_SHIELD: u32 = 50;
pub const MECH_MAX_SHIELD: u32 = 50;
pub const INITIAL_UPGRADE_LEVEL: u8 = 1;

// ===== Movement and Distances =====
pub const RESOURCE_PICKUP_DISTANCE: f32 = 1.5; // tiles
pub const MECH_DOOR_ENTRY_DISTANCE: f32 = 0.8; // tiles
pub const LADDER_INTERACTION_DISTANCE: f32 = 0.3; // tiles
pub const MECH_COLLISION_DISTANCE: f32 = 5.0; // tiles for resource deposit

// ===== Combat =====
pub const LASER_BASE_DAMAGE: u32 = 10;
pub const LASER_DAMAGE_PER_LEVEL: u32 = 10;
pub const PROJECTILE_BASE_DAMAGE: u32 = 15;
pub const PROJECTILE_DAMAGE_PER_LEVEL: u32 = 15;
pub const PROJECTILE_BASE_SPEED: f32 = 300.0; // pixels per second
pub const PROJECTILE_LIFETIME: f32 = 5.0; // seconds
pub const SHIELD_BOOST_AMOUNT: u32 = 10;
pub const SHIELD_PER_LEVEL: u32 = 25;

// ===== Engine and Speed =====
pub const MECH_BASE_SPEED: f32 = 2.0; // tiles per second
pub const MECH_SPEED_PER_LEVEL: f32 = 0.5; // additional tiles per second
pub const MECH_DEBUG_SPEED: f32 = 1.0; // tiles per second (slow debug speed)
pub const CONTINUOUS_MOVEMENT_DELTA: f32 = 0.016; // ~60fps frame time
pub const PLAYER_MOVE_SPEED: f32 = 4.5; // tiles per second

// ===== Collision Radii =====
pub const PLAYER_COLLISION_RADIUS: f32 = 0.4; // tiles
pub const MECH_COLLISION_RADIUS: f32 = 5.0; // tiles (10x10 tiles = 5 tile radius)
pub const RESOURCE_COLLISION_RADIUS: f32 = 0.3; // tiles
pub const PROJECTILE_COLLISION_RADIUS: f32 = 0.2; // tiles
pub const WEAPON_MAX_RANGE: f32 = 50.0; // tiles

// ===== Collision Behavior =====
pub const RUN_OVER_MIN_VELOCITY: f32 = 1.0; // tiles per second - minimum mech speed to kill players
pub const MECH_SEPARATION_FORCE: f32 = 2.0; // force applied to separate overlapping mechs
pub const PLAYER_PUSH_DISTANCE: f32 = 0.5; // tiles - how far to push players away from mechs
pub const COLLISION_EPSILON: f32 = 0.001; // small value to prevent floating point issues

// ===== Repairs and Upgrades =====
pub const REPAIR_HP_PER_SCRAP: u32 = 20;

// ===== Game Balance =====
pub const MAX_TEAM_SIZE_DIFFERENCE: usize = 1;
pub const MAX_UPGRADE_LEVEL: u8 = 5;

// ===== Resource Costs =====
pub mod upgrade_costs {
    use crate::types::ResourceType;

    pub const LASER_UPGRADE: &[(ResourceType, usize)] = &[
        (ResourceType::ScrapMetal, 2),
        (ResourceType::ComputerComponents, 1),
    ];

    pub const PROJECTILE_UPGRADE: &[(ResourceType, usize)] = &[(ResourceType::ScrapMetal, 3)];

    pub const SHIELD_UPGRADE: &[(ResourceType, usize)] =
        &[(ResourceType::Batteries, 2), (ResourceType::Wiring, 1)];

    pub const ENGINE_UPGRADE: &[(ResourceType, usize)] = &[
        (ResourceType::ComputerComponents, 2),
        (ResourceType::Wiring, 2),
    ];
}

// ===== Initial Resource Spawns =====
pub const INITIAL_RESOURCE_SPAWNS: &[(i32, i32)] =
    &[(40, 30), (60, 30), (30, 60), (70, 60), (50, 50)];

// ===== Mech Interior Layout =====
pub const LADDER_POSITIONS: &[(i32, i32)] = &[
    (2, 2),
    // Second ladder position calculated as (FLOOR_WIDTH_TILES - 3, 2) = (7, 2)
];

pub const STATION_POSITIONS: &[&[(i32, i32)]] = &[
    // Floor 0
    &[(5, 3), (10, 3), (15, 3)],
    // Floor 1
    &[(5, 3), (10, 3), (15, 3)],
    // Floor 2
    &[(8, 3)],
];
