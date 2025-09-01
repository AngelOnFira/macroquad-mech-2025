use uuid::Uuid;

/// Generate a new UUID v4. Only available on non-WASM targets.
#[cfg(not(target_arch = "wasm32"))]
pub fn new_uuid() -> Uuid {
    Uuid::new_v4()
}

/// Placeholder for WASM builds - should never be called
#[cfg(target_arch = "wasm32")]
pub fn new_uuid() -> Uuid {
    panic!("UUID generation is not available in WASM builds. UUIDs should be generated server-side only.");
}
