use serde::{Deserialize, Serialize};

/// Debug settings that can be persisted across sessions
#[derive(Serialize, Deserialize, Clone)]
pub struct DebugSettings {
    // UI panel visibility
    pub show_performance: bool,
    pub show_server_state: bool,
    pub show_mini_map: bool,
    pub show_network: bool,
    pub show_rendering_toggles: bool,
    pub show_spatial_debug: bool,

    // Spatial debug controls
    pub spatial_debug_enabled: bool,
    pub show_coordinate_transforms: bool,
    pub show_mech_bounds: bool,
    pub show_door_positions: bool,
    pub show_coordinate_grid: bool,
    pub show_floor_offsets: bool,

    // Mech control debug panel
    pub show_mech_controls: bool,

    // Rendering toggles
    pub render_mechs: bool,
    pub render_players: bool,
    pub render_resources: bool,
    pub render_projectiles: bool,
    pub render_effects: bool,
    pub render_ui: bool,
    pub render_fog: bool,
    pub render_tiles: bool,
    pub render_stations: bool,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            // UI panel defaults (match debug_overlay.rs defaults)
            show_performance: false,
            show_server_state: true,
            show_mini_map: false,
            show_network: true,
            show_rendering_toggles: true,
            show_spatial_debug: true,

            // Mech control defaults
            show_mech_controls: false,

            // Spatial debug defaults
            spatial_debug_enabled: false,
            show_coordinate_transforms: false,
            show_mech_bounds: true,
            show_door_positions: true,
            show_coordinate_grid: false,
            show_floor_offsets: true,

            // All rendering enabled by default
            render_mechs: true,
            render_players: true,
            render_resources: true,
            render_projectiles: true,
            render_effects: true,
            render_ui: true,
            render_fog: true,
            render_tiles: true,
            render_stations: true,
        }
    }
}

// FFI functions for WebAssembly storage
#[cfg(target_arch = "wasm32")]
extern "C" {
    fn js_settings_save(key_ptr: *const u8, key_len: usize, value_ptr: *const u8, value_len: usize) -> i32;
    fn js_settings_load(key_ptr: *const u8, key_len: usize, buffer_ptr: *mut u8, buffer_len: usize) -> i32;
    fn js_settings_exists(key_ptr: *const u8, key_len: usize) -> i32;
}

pub struct SettingsManager {
    settings: DebugSettings,
}

impl SettingsManager {
    pub fn new() -> Self {
        let settings = Self::load_settings().unwrap_or_default();
        Self { settings }
    }

    pub fn get_settings(&self) -> &DebugSettings {
        &self.settings
    }

    pub fn update_settings(&mut self, settings: DebugSettings) {
        self.settings = settings;
        self.save_settings();
    }

    #[cfg(target_arch = "wasm32")]
    fn load_settings() -> Option<DebugSettings> {
        use std::slice;

        let key = "debug_settings";
        let key_bytes = key.as_bytes();
        let mut buffer = [0u8; 4096]; // Should be enough for settings JSON
        
        unsafe {
            let result = js_settings_load(
                key_bytes.as_ptr(),
                key_bytes.len(),
                buffer.as_mut_ptr(),
                buffer.len(),
            );
            
            if result > 0 {
                let data = slice::from_raw_parts(buffer.as_ptr(), result as usize);
                let json_str = std::str::from_utf8(data).ok()?;
                serde_json::from_str(json_str).ok()
            } else {
                None
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_settings() -> Option<DebugSettings> {
        // For native builds, we don't persist settings
        // Could be extended to use a config file later
        None
    }

    #[cfg(target_arch = "wasm32")]
    fn save_settings(&self) {
        if let Ok(json) = serde_json::to_string(&self.settings) {
            let key = "debug_settings";
            let key_bytes = key.as_bytes();
            let json_bytes = json.as_bytes();
            
            unsafe {
                js_settings_save(
                    key_bytes.as_ptr(),
                    key_bytes.len(),
                    json_bytes.as_ptr(),
                    json_bytes.len(),
                );
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_settings(&self) {
        // No-op for native builds
    }

    #[cfg(target_arch = "wasm32")]
    pub fn settings_exist() -> bool {
        let key = "debug_settings";
        let key_bytes = key.as_bytes();
        
        unsafe {
            js_settings_exists(key_bytes.as_ptr(), key_bytes.len()) != 0
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn settings_exist() -> bool {
        false
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}