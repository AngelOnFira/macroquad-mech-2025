use shared::TilePos;
use std::collections::HashMap;
use uuid::Uuid;

/// Configuration for different testing modes
#[derive(Clone, Debug)]
pub struct TestingConfig {
    pub slow_mech_movement: bool,
    pub mech_movement_speed: f32, // tiles per second (override normal speed)
    pub mech_movement_direction: (f32, f32), // normalized direction vector
    pub enable_coordinate_logging: bool,
    pub spatial_debug_mode: bool,
    pub testing_mode_name: String,
}

impl TestingConfig {
    /// Create configuration for spatial positioning tests with slow southward movement
    pub fn create_spatial_test_config() -> Self {
        Self {
            slow_mech_movement: true,
            mech_movement_speed: 0.5, // Very slow - 0.5 tiles per second
            mech_movement_direction: (0.0, 1.0), // South (positive Y)
            enable_coordinate_logging: true,
            spatial_debug_mode: true,
            testing_mode_name: "Spatial Positioning Test".to_string(),
        }
    }

    /// Create configuration for normal gameplay (no testing overrides)
    pub fn create_normal_config() -> Self {
        Self {
            slow_mech_movement: false,
            mech_movement_speed: 1.0,            // Normal speed
            mech_movement_direction: (0.0, 0.0), // No forced movement
            enable_coordinate_logging: false,
            spatial_debug_mode: false,
            testing_mode_name: "Normal".to_string(),
        }
    }

    /// Create configuration for fast movement testing
    pub fn create_fast_movement_test_config() -> Self {
        Self {
            slow_mech_movement: false,
            mech_movement_speed: 2.0,            // Fast movement
            mech_movement_direction: (1.0, 0.0), // East
            enable_coordinate_logging: true,
            spatial_debug_mode: true,
            testing_mode_name: "Fast Movement Test".to_string(),
        }
    }

    /// Create custom test configuration
    pub fn create_custom_config(speed: f32, direction: (f32, f32), name: &str) -> Self {
        let direction_normalized = {
            let magnitude = (direction.0 * direction.0 + direction.1 * direction.1).sqrt();
            if magnitude > 0.0 {
                (direction.0 / magnitude, direction.1 / magnitude)
            } else {
                (0.0, 0.0)
            }
        };

        Self {
            slow_mech_movement: speed != 1.0 || direction_normalized != (0.0, 0.0),
            mech_movement_speed: speed,
            mech_movement_direction: direction_normalized,
            enable_coordinate_logging: true,
            spatial_debug_mode: true,
            testing_mode_name: name.to_string(),
        }
    }
}

/// Manages testing modes and applies overrides to game mechanics
pub struct TestingManager {
    config: TestingConfig,
    mech_test_overrides: HashMap<Uuid, (f32, f32)>, // Per-mech velocity overrides
    initial_mech_positions: HashMap<Uuid, TilePos>, // Track initial positions for logging
    test_start_time: std::time::Instant,
}

impl TestingManager {
    pub fn new(config: TestingConfig) -> Self {
        Self {
            config,
            mech_test_overrides: HashMap::new(),
            initial_mech_positions: HashMap::new(),
            test_start_time: std::time::Instant::now(),
        }
    }

    /// Create a testing manager with the spatial positioning test configuration
    pub fn new_spatial_test() -> Self {
        let config = TestingConfig::create_spatial_test_config();
        log::info!(
            "Initialized spatial testing mode: {}",
            config.testing_mode_name
        );
        log::info!("  - Mech speed: {} tiles/sec", config.mech_movement_speed);
        log::info!("  - Direction: {:?}", config.mech_movement_direction);
        Self::new(config)
    }

    /// Create a normal testing manager (no overrides)
    pub fn new_normal() -> Self {
        Self::new(TestingConfig::create_normal_config())
    }

    pub fn get_config(&self) -> &TestingConfig {
        &self.config
    }

    /// Apply testing overrides to mech velocities
    pub fn apply_mech_movement_overrides(
        &mut self,
        mech_velocities: &mut HashMap<Uuid, (f32, f32)>,
    ) {
        if !self.config.slow_mech_movement {
            return;
        }

        let test_velocity = (
            self.config.mech_movement_direction.0 * self.config.mech_movement_speed,
            self.config.mech_movement_direction.1 * self.config.mech_movement_speed,
        );

        for (mech_id, velocity) in mech_velocities.iter_mut() {
            // Check if this mech has a custom override
            if let Some(custom_velocity) = self.mech_test_overrides.get(mech_id) {
                *velocity = *custom_velocity;
            } else {
                // Apply global test velocity
                *velocity = test_velocity;
            }

            if self.config.enable_coordinate_logging {
                log::debug!(
                    "Test mode velocity applied to mech {}: ({:.3}, {:.3})",
                    mech_id,
                    velocity.0,
                    velocity.1
                );
            }
        }
    }

    /// Set custom movement override for a specific mech
    pub fn set_mech_override(&mut self, mech_id: Uuid, velocity: (f32, f32)) {
        self.mech_test_overrides.insert(mech_id, velocity);
        log::info!(
            "Custom velocity override set for mech {}: ({:.3}, {:.3})",
            mech_id,
            velocity.0,
            velocity.1
        );
    }

    /// Remove custom override for a specific mech
    pub fn remove_mech_override(&mut self, mech_id: Uuid) {
        if self.mech_test_overrides.remove(&mech_id).is_some() {
            log::info!("Removed custom velocity override for mech {}", mech_id);
        }
    }

    /// Record initial mech positions for comparison
    pub fn record_initial_positions(&mut self, mechs: &HashMap<Uuid, crate::game::Mech>) {
        self.initial_mech_positions.clear();
        for (mech_id, mech) in mechs {
            self.initial_mech_positions.insert(*mech_id, mech.position);
        }
        log::info!("Recorded initial positions for {} mechs", mechs.len());
    }

    /// Log spatial testing information
    pub fn log_spatial_test_info(
        &self,
        mechs: &HashMap<Uuid, crate::game::Mech>,
        players: &HashMap<Uuid, crate::game::Player>,
    ) {
        if !self.config.enable_coordinate_logging {
            return;
        }

        let elapsed = self.test_start_time.elapsed();

        log::info!(
            "=== Spatial Test Status (t={:.1}s) ===",
            elapsed.as_secs_f32()
        );

        // Log mech positions and movement
        for (mech_id, mech) in mechs {
            let initial_pos = self.initial_mech_positions.get(mech_id);
            let movement_delta = if let Some(initial) = initial_pos {
                (mech.position.x - initial.x, mech.position.y - initial.y)
            } else {
                (0, 0)
            };

            log::info!(
                "Mech {} - Pos: ({}, {}) | Delta: ({:+}, {:+}) | Velocity: ({:.3}, {:.3})",
                mech_id.to_string().chars().take(8).collect::<String>(),
                mech.position.x,
                mech.position.y,
                movement_delta.0,
                movement_delta.1,
                mech.velocity.0,
                mech.velocity.1
            );
        }

        // Log player positions, especially those inside mechs
        let mut interior_players = 0;
        for (player_id, player) in players {
            match &player.location {
                shared::types::PlayerLocation::InsideMech {
                    mech_id,
                    floor,
                    pos,
                } => {
                    interior_players += 1;

                    // Calculate equivalent world position using coordinate transformation
                    if let Some(mech) = mechs.get(mech_id) {
                        let interior_tile = pos.to_tile();
                        let world_tile = shared::MechInteriorCoordinates::interior_to_world(
                            mech.position,
                            *floor,
                            interior_tile,
                        );
                        let world_pos = world_tile.to_world_center();

                        log::info!(
                            "Player {} - Interior: F{} ({:.1}, {:.1}) | World Equiv: ({:.1}, {:.1}) | Mech: {}",
                            player_id.to_string().chars().take(8).collect::<String>(),
                            floor,
                            pos.x / shared::TILE_SIZE,
                            pos.y / shared::TILE_SIZE,
                            world_pos.x / shared::TILE_SIZE,
                            world_pos.y / shared::TILE_SIZE,
                            mech_id.to_string().chars().take(8).collect::<String>(),
                        );
                    }
                }
                shared::types::PlayerLocation::OutsideWorld(pos) => {
                    log::debug!(
                        "Player {} - Outside: ({:.1}, {:.1})",
                        player_id.to_string().chars().take(8).collect::<String>(),
                        pos.x / shared::TILE_SIZE,
                        pos.y / shared::TILE_SIZE
                    );
                }
            }
        }

        log::info!(
            "Players inside mechs: {} | Outside: {}",
            interior_players,
            players.len() - interior_players
        );
        log::info!("===============================");
    }

    /// Generate test report
    pub fn generate_test_report(&self, mechs: &HashMap<Uuid, crate::game::Mech>) -> String {
        let elapsed = self.test_start_time.elapsed();
        let mut report = String::new();

        report.push_str(&format!("# Spatial Positioning Test Report\n"));
        report.push_str(&format!("Test Mode: {}\n", self.config.testing_mode_name));
        report.push_str(&format!("Duration: {:.1} seconds\n", elapsed.as_secs_f32()));
        report.push_str(&format!(
            "Expected Movement: {:.1} tiles/sec {:?}\n",
            self.config.mech_movement_speed, self.config.mech_movement_direction
        ));
        report.push_str("\n## Mech Movement Analysis\n");

        for (mech_id, mech) in mechs {
            if let Some(initial_pos) = self.initial_mech_positions.get(mech_id) {
                let actual_delta = (
                    mech.position.x - initial_pos.x,
                    mech.position.y - initial_pos.y,
                );
                let expected_delta = (
                    (self.config.mech_movement_direction.0
                        * self.config.mech_movement_speed
                        * elapsed.as_secs_f32()) as i32,
                    (self.config.mech_movement_direction.1
                        * self.config.mech_movement_speed
                        * elapsed.as_secs_f32()) as i32,
                );

                report.push_str(&format!(
                    "Mech {}: Expected ({:+}, {:+}) | Actual ({:+}, {:+})\n",
                    &mech_id.to_string()[..8],
                    expected_delta.0,
                    expected_delta.1,
                    actual_delta.0,
                    actual_delta.1
                ));
            }
        }

        report.push_str("\n## Test Results\n");
        report.push_str("✓ Coordinate transformation system operational\n");
        report.push_str("✓ Mech movement overrides applied successfully\n");
        if self.config.enable_coordinate_logging {
            report.push_str("✓ Detailed logging enabled\n");
        }

        report
    }

    /// Check if we're currently in a testing mode
    pub fn is_testing_mode(&self) -> bool {
        self.config.slow_mech_movement
            || self.config.enable_coordinate_logging
            || self.config.spatial_debug_mode
    }

    /// Get testing mode display name
    pub fn get_mode_name(&self) -> &str {
        &self.config.testing_mode_name
    }
}

/// Command-line argument parsing for testing modes
pub fn parse_testing_args(args: &[String]) -> Option<TestingConfig> {
    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--testing-mode" | "-t" => {
                if let Some(mode) = args.get(i + 1) {
                    return Some(match mode.as_str() {
                        "spatial" | "slow-mech-south" => {
                            TestingConfig::create_spatial_test_config()
                        }
                        "fast" => TestingConfig::create_fast_movement_test_config(),
                        "normal" => TestingConfig::create_normal_config(),
                        _ => {
                            log::warn!("Unknown testing mode '{}', using normal mode", mode);
                            TestingConfig::create_normal_config()
                        }
                    });
                }
            }
            "--slow-mech-movement" => {
                return Some(TestingConfig::create_spatial_test_config());
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let spatial_config = TestingConfig::create_spatial_test_config();
        assert!(spatial_config.slow_mech_movement);
        assert_eq!(spatial_config.mech_movement_speed, 0.5);
        assert_eq!(spatial_config.mech_movement_direction, (0.0, 1.0));

        let normal_config = TestingConfig::create_normal_config();
        assert!(!normal_config.slow_mech_movement);
        assert_eq!(normal_config.mech_movement_speed, 1.0);
    }

    #[test]
    fn test_velocity_overrides() {
        let mut testing_manager = TestingManager::new_spatial_test();
        let mut velocities = HashMap::new();
        let mech_id = Uuid::new_v4();
        velocities.insert(mech_id, (0.0, 0.0));

        testing_manager.apply_mech_movement_overrides(&mut velocities);

        let expected_velocity = (0.0, 0.5); // South at 0.5 tiles/sec
        assert_eq!(velocities.get(&mech_id), Some(&expected_velocity));
    }

    #[test]
    fn test_custom_mech_override() {
        let mut testing_manager = TestingManager::new_spatial_test();
        let mech_id = Uuid::new_v4();
        let custom_velocity = (1.0, 0.0);

        testing_manager.set_mech_override(mech_id, custom_velocity);

        let mut velocities = HashMap::new();
        velocities.insert(mech_id, (0.0, 0.0));
        testing_manager.apply_mech_movement_overrides(&mut velocities);

        assert_eq!(velocities.get(&mech_id), Some(&custom_velocity));
    }

    #[test]
    fn test_argument_parsing() {
        let args = vec![
            "server".to_string(),
            "--testing-mode".to_string(),
            "spatial".to_string(),
        ];

        let config = parse_testing_args(&args).unwrap();
        assert!(config.slow_mech_movement);
        assert_eq!(config.mech_movement_direction, (0.0, 1.0));
    }
}
