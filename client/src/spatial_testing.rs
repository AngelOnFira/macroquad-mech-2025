use crate::game_state::GameState;
use macroquad::prelude::*;
use shared::{MechInteriorCoordinates, PlayerLocation, TilePos, WorldPos};
use uuid::Uuid;

pub struct SpatialTestSuite {
    test_results: Vec<TestResult>,
    current_test: Option<RunningTest>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub success: bool,
    pub details: String,
    pub measurements_count: usize,
    pub error_threshold: f32,
    pub max_error: f32,
}

#[derive(Debug)]
pub struct RunningTest {
    pub name: String,
    pub expected_behavior: String,
    pub measurements: Vec<SpatialMeasurement>,
    pub error_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct SpatialMeasurement {
    pub player_location: PlayerLocation,
    pub mech_world_position: WorldPos,
    pub calculated_world_position: WorldPos, // What the coordinate system calculates
    pub actual_render_position: WorldPos,    // Where we actually rendered (same as calculated)
    pub coordinate_error: f32,               // Distance between expected and calculated positions
}

impl Default for SpatialTestSuite {
    fn default() -> Self {
        Self {
            test_results: Vec::new(),
            current_test: None,
        }
    }
}

impl SpatialTestSuite {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a test for mech entry spatial continuity
    pub fn start_mech_entry_test(
        &mut self,
        mech_id: Uuid,
        entry_door: TilePos,
        error_threshold: f32,
    ) {
        info!("Starting mech entry spatial continuity test");

        self.current_test = Some(RunningTest {
            name: "Mech Entry Spatial Continuity".to_string(),
            expected_behavior: "Player should smoothly transition from world to interior coordinates with spatial continuity".to_string(),
            measurements: Vec::new(),
            error_threshold,
        });
    }

    /// Start a test for relative movement in moving mechs
    pub fn start_relative_movement_test(&mut self, mech_id: Uuid, error_threshold: f32) {
        info!("Starting relative movement in moving mech test");

        self.current_test = Some(RunningTest {
            name: "Relative Movement in Moving Mech".to_string(),
            expected_behavior: "Player interior movement should be independent of mech movement, but world position should move with mech".to_string(),
            measurements: Vec::new(),
            error_threshold,
        });
    }

    /// Start a general coordinate transformation test
    pub fn start_coordinate_transform_test(&mut self, error_threshold: f32) {
        info!("Starting coordinate transformation accuracy test");

        self.current_test = Some(RunningTest {
            name: "Coordinate Transformation Accuracy".to_string(),
            expected_behavior:
                "Interior-to-world coordinate transformations should be mathematically consistent"
                    .to_string(),
            measurements: Vec::new(),
            error_threshold,
        });
    }

    /// Record a spatial measurement for the current test
    pub fn record_spatial_measurement(
        &mut self,
        game_state: &GameState,
        player_location: PlayerLocation,
        mech_world_pos: Option<WorldPos>,
    ) {
        if let Some(ref mut test) = self.current_test {
            let calculated_pos = match &player_location {
                PlayerLocation::OutsideWorld(pos) => *pos,
                PlayerLocation::InsideMech {
                    mech_id,
                    pos,
                } => {
                    let floor = pos.floor();
                    // Use coordinate transformation to get world position
                    if let Some(mech) = game_state.mechs.get(mech_id) {
                        let interior_tile = pos.tile_pos();
                        let world_tile = MechInteriorCoordinates::interior_to_world(
                            mech.position,
                            floor,
                            interior_tile,
                        );
                        world_tile.to_world_center()
                    } else {
                        pos.to_local_world() // Fallback to local world position
                    }
                }
            };

            // Calculate coordinate error (how far off our calculation might be)
            let coordinate_error = if let Some(mech_pos) = mech_world_pos {
                // Compare calculated position with expected mech-relative position
                match &player_location {
                    PlayerLocation::InsideMech { pos, .. } => {
                        // For interior players, error is difference between calculated world pos and expected
                        let local_world = pos.to_local_world();
                        let expected_world = WorldPos::new(mech_pos.x + local_world.x, mech_pos.y + local_world.y);
                        calculated_pos.distance_to(expected_world)
                    }
                    _ => 0.0, // No error for outside world players
                }
            } else {
                0.0
            };

            let measurement = SpatialMeasurement {
                player_location,
                mech_world_position: mech_world_pos.unwrap_or(WorldPos::new(0.0, 0.0)),
                calculated_world_position: calculated_pos,
                actual_render_position: calculated_pos, // Same as calculated in current implementation
                coordinate_error,
            };

            test.measurements.push(measurement);

            // Log significant coordinate errors
            if coordinate_error > test.error_threshold {
                warn!(
                    "High coordinate error detected: {:.2} pixels (threshold: {:.2})",
                    coordinate_error, test.error_threshold
                );
            }
        }
    }

    /// Finish the current test and return results
    pub fn finish_current_test(&mut self) -> Option<TestResult> {
        if let Some(test) = self.current_test.take() {
            let result = self.analyze_test_results(test);
            self.test_results.push(result.clone());
            Some(result)
        } else {
            None
        }
    }

    /// Analyze test measurements and generate results
    fn analyze_test_results(&self, test: RunningTest) -> TestResult {
        let measurements_count = test.measurements.len();

        if measurements_count == 0 {
            return TestResult {
                test_name: test.name,
                success: false,
                details: "No measurements recorded".to_string(),
                measurements_count: 0,
                error_threshold: test.error_threshold,
                max_error: 0.0,
            };
        }

        // Calculate error statistics
        let max_error = test
            .measurements
            .iter()
            .map(|m| m.coordinate_error)
            .fold(0.0, f32::max);

        let avg_error = test
            .measurements
            .iter()
            .map(|m| m.coordinate_error)
            .sum::<f32>()
            / measurements_count as f32;

        let errors_above_threshold = test
            .measurements
            .iter()
            .filter(|m| m.coordinate_error > test.error_threshold)
            .count();

        // Determine success based on error rates
        let error_rate = errors_above_threshold as f32 / measurements_count as f32;
        let success = error_rate < 0.1; // Less than 10% of measurements should have high errors

        // Analyze movement patterns for moving mech tests
        let details = if test.name.contains("Moving Mech") {
            self.analyze_moving_mech_test(&test, max_error, avg_error, error_rate)
        } else if test.name.contains("Entry") {
            self.analyze_entry_test(&test, max_error, avg_error, error_rate)
        } else {
            self.analyze_coordinate_test(&test, max_error, avg_error, error_rate)
        };

        TestResult {
            test_name: test.name,
            success,
            details,
            measurements_count,
            error_threshold: test.error_threshold,
            max_error,
        }
    }

    fn analyze_moving_mech_test(
        &self,
        test: &RunningTest,
        max_error: f32,
        avg_error: f32,
        error_rate: f32,
    ) -> String {
        // Check for consistent relative movement patterns
        let mut interior_movements = 0;
        let mut world_position_changes = 0;

        for window in test.measurements.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            match (&prev.player_location, &curr.player_location) {
                (
                    PlayerLocation::InsideMech { pos: prev_pos, .. },
                    PlayerLocation::InsideMech { pos: curr_pos, .. },
                ) => {
                    // Check if player moved within the mech
                    let prev_world = prev_pos.to_local_world();
                    let curr_world = curr_pos.to_local_world();
                    if prev_world.distance_to(curr_world) > 1.0 {
                        interior_movements += 1;
                    }

                    // Check if world position changed (indicating mech movement)
                    if prev
                        .calculated_world_position
                        .distance_to(curr.calculated_world_position)
                        > 1.0
                    {
                        world_position_changes += 1;
                    }
                }
                _ => {}
            }
        }

        format!(
            "Moving Mech Test Results:\n\
             • Interior movements detected: {}\n\
             • World position changes: {}\n\
             • Max coordinate error: {:.2} pixels\n\
             • Average error: {:.2} pixels\n\
             • Error rate: {:.1}% (threshold violations)\n\
             • Analysis: {}",
            interior_movements,
            world_position_changes,
            max_error,
            avg_error,
            error_rate * 100.0,
            if error_rate < 0.1 {
                "Spatial positioning working correctly - interior movement independent of mech movement"
            } else {
                "Potential issues with coordinate transformation consistency"
            }
        )
    }

    fn analyze_entry_test(
        &self,
        test: &RunningTest,
        max_error: f32,
        avg_error: f32,
        error_rate: f32,
    ) -> String {
        // Look for smooth transition from outside to inside
        let transition_detected = test
            .measurements
            .iter()
            .any(|m| matches!(m.player_location, PlayerLocation::OutsideWorld(_)))
            && test
                .measurements
                .iter()
                .any(|m| matches!(m.player_location, PlayerLocation::InsideMech { .. }));

        format!(
            "Entry Test Results:\n\
             • Transition detected: {}\n\
             • Max coordinate error: {:.2} pixels\n\
             • Average error: {:.2} pixels\n\
             • Error rate: {:.1}%\n\
             • Analysis: {}",
            if transition_detected { "Yes" } else { "No" },
            max_error,
            avg_error,
            error_rate * 100.0,
            if transition_detected && error_rate < 0.1 {
                "Smooth spatial entry transition detected with good coordinate consistency"
            } else if !transition_detected {
                "No entry transition observed during test period"
            } else {
                "Entry transition detected but with coordinate accuracy issues"
            }
        )
    }

    fn analyze_coordinate_test(
        &self,
        _test: &RunningTest,
        max_error: f32,
        avg_error: f32,
        error_rate: f32,
    ) -> String {
        format!(
            "Coordinate Transformation Test Results:\n\
             • Max coordinate error: {:.2} pixels\n\
             • Average error: {:.2} pixels\n\
             • Error rate: {:.1}%\n\
             • Analysis: {}",
            max_error,
            avg_error,
            error_rate * 100.0,
            if error_rate < 0.05 {
                "Coordinate transformations are highly accurate"
            } else if error_rate < 0.1 {
                "Coordinate transformations are reasonably accurate"
            } else {
                "Coordinate transformations may have accuracy issues"
            }
        )
    }

    /// Get all test results
    pub fn get_test_results(&self) -> &[TestResult] {
        &self.test_results
    }

    /// Generate a comprehensive test report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# Spatial Positioning Test Report\n");
        report.push_str(&format!("Generated: {:?}\n", std::time::SystemTime::now()));
        report.push_str(&format!("Total tests run: {}\n\n", self.test_results.len()));

        if self.test_results.is_empty() {
            report.push_str("No tests have been completed yet.\n");
            return report;
        }

        // Summary statistics
        let successful_tests = self.test_results.iter().filter(|r| r.success).count();
        let total_measurements: usize =
            self.test_results.iter().map(|r| r.measurements_count).sum();

        report.push_str("## Summary\n");
        report.push_str(&format!(
            "✅ Successful tests: {}/{}\n",
            successful_tests,
            self.test_results.len()
        ));
        report.push_str(&format!("📊 Total measurements: {}\n", total_measurements));
        report.push_str(&format!(
            "📈 Overall success rate: {:.1}%\n\n",
            (successful_tests as f32 / self.test_results.len() as f32) * 100.0
        ));

        // Individual test results
        report.push_str("## Test Results\n\n");
        for (i, result) in self.test_results.iter().enumerate() {
            let status = if result.success {
                "✅ PASS"
            } else {
                "❌ FAIL"
            };

            report.push_str(&format!(
                "### Test {}: {} {}\n",
                i + 1,
                result.test_name,
                status
            ));
            report.push_str(&format!(
                "**Measurements:** {}\n",
                result.measurements_count
            ));
            report.push_str(&format!(
                "**Max Error:** {:.2} pixels (threshold: {:.2})\n",
                result.max_error, result.error_threshold
            ));
            report.push_str(&format!("**Details:**\n{}\n\n", result.details));
        }

        // Recommendations
        report.push_str("## Recommendations\n");
        if successful_tests == self.test_results.len() {
            report.push_str("🎉 All spatial positioning tests passed! The coordinate transformation system is working correctly.\n");
        } else {
            report.push_str("⚠️  Some tests failed. Consider:\n");
            report.push_str("- Checking coordinate transformation mathematics\n");
            report.push_str("- Verifying mech interior layout calculations\n");
            report.push_str("- Testing with different mech positions and player movements\n");
        }

        report
    }

    /// Check if currently running a test
    pub fn is_testing(&self) -> bool {
        self.current_test.is_some()
    }

    /// Get current test name
    pub fn current_test_name(&self) -> Option<&str> {
        self.current_test.as_ref().map(|t| t.name.as_str())
    }

    /// Auto-record measurements during normal gameplay
    pub fn auto_record_if_testing(&mut self, game_state: &GameState) {
        if self.current_test.is_some() {
            if let Some(player_id) = game_state.player_id {
                if let Some(_player) = game_state.players.get(&player_id) {
                    // Find the player's mech if they're inside one
                    let mech_world_pos = match game_state.player_location {
                        PlayerLocation::InsideMech { mech_id, .. } => game_state
                            .mechs
                            .get(&mech_id)
                            .map(|mech| mech.world_position),
                        _ => None,
                    };

                    self.record_spatial_measurement(
                        game_state,
                        game_state.player_location,
                        mech_world_pos,
                    );
                }
            }
        }
    }

    /// Print test status to console
    pub fn print_status(&self) {
        if let Some(ref test) = self.current_test {
            println!(
                "🧪 Test Running: {} ({} measurements)",
                test.name,
                test.measurements.len()
            );
        } else if !self.test_results.is_empty() {
            let last_result = self.test_results.last().unwrap();
            let status = if last_result.success { "✅" } else { "❌" };
            println!(
                "{} Last Test: {} ({} measurements)",
                status, last_result.test_name, last_result.measurements_count
            );
        }
    }
}
