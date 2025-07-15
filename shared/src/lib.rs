pub mod messages;
pub mod types;
pub mod constants;
pub mod balance;
pub mod render_constants;
pub mod network_constants;
pub mod errors;
pub mod validation;
pub mod network;
pub mod mech_layout;
pub mod spatial;
pub mod coordinates;
pub mod stations;
pub mod uuid_gen;

// Object pool is only needed server-side (uses Uuid::new_v4)
#[cfg(not(target_arch = "wasm32"))]
pub mod object_pool;

pub use messages::*;
pub use types::*;
pub use constants::*;
pub use balance::*;
pub use render_constants::*;
pub use network_constants::*;
pub use errors::*;
pub use validation::*;
pub use network::*;
pub use mech_layout::*;
pub use spatial::*;
pub use coordinates::*;
pub use stations::*;

// Only export object_pool for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
pub use object_pool::*;