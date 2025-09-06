pub mod balance;
pub mod collision;
pub mod components;
pub mod constants;
pub mod coordinates;
pub mod errors;
pub mod mech_coordinates;
pub mod mech_layout;
pub mod messages;
pub mod network;
pub mod network_constants;
pub mod render_constants;
pub mod spatial;
pub mod stations;
pub mod tile_entity;
pub mod tile_math;
pub mod types;
pub mod uuid_gen;
pub mod validation;
pub mod vision;

// Object pool is only needed server-side (uses Uuid::new_v4)
#[cfg(not(target_arch = "wasm32"))]
pub mod object_pool;

pub use balance::*;
pub use collision::*;
pub use constants::*;
pub use coordinates::*;
pub use errors::*;
pub use mech_coordinates::*;
pub use mech_layout::*;
pub use messages::*;
pub use network::*;
pub use network_constants::*;
pub use render_constants::*;
pub use spatial::*;
pub use tile_math::*;
pub use types::*;
pub use validation::*;
// Export stations module types selectively to avoid conflicts
pub use stations::{
    ButtonDefinition, MechUpgradeType, StationAction, StationActionContext, StationActionResult,
    StationDefinition, StationEffect, StationInstance, StationRegistry, WeaponType,
};
// Station component is exported from components module

// Only export object_pool for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
pub use object_pool::*;

// Export new tile-entity system types
pub use components::*;
pub use tile_entity::*;
pub use vision::*;
