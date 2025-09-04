#[cfg(debug_assertions)]
use crate::game_state::GameState;
#[cfg(debug_assertions)]
use egui::*;
#[cfg(debug_assertions)]
use macroquad::prelude::get_fps;
#[cfg(debug_assertions)]
use shared::{tile_entity::TileVisual, types::*, StationType};
#[cfg(debug_assertions)]
use std::collections::VecDeque;

#[cfg(debug_assertions)]
pub struct DebugOverlay {
    // Performance tracking
    frame_times: VecDeque<(f32, f32)>, // (elapsed_time, frame_time_ms)
    fps_history: VecDeque<(f32, f32)>, // (elapsed_time, fps)
    elapsed_time: f32,                 // Total elapsed time since start

    // Smoothing for stability
    fps_smoothing_buffer: VecDeque<f32>, // Raw FPS values for smoothing

    // UI state
    show_performance: bool,
    show_server_state: bool,
    show_mini_map: bool,
    show_network: bool,
    show_rendering_toggles: bool,

    // Server state tracking
    last_server_message: String,
    message_history: VecDeque<String>,
    message_counter: u32,

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

    // ASCII view settings
    ascii_grid_size: (usize, usize),
    mini_map_zoom: f32,
}

#[cfg(debug_assertions)]
impl DebugOverlay {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            fps_history: VecDeque::with_capacity(120),
            elapsed_time: 0.0,
            fps_smoothing_buffer: VecDeque::with_capacity(10), // 10-frame smoothing

            show_performance: false,
            show_server_state: true,
            show_mini_map: false,
            show_network: true,
            show_rendering_toggles: true,

            last_server_message: String::new(),
            message_history: VecDeque::with_capacity(20),
            message_counter: 0,

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

            ascii_grid_size: (40, 20),
            mini_map_zoom: 1.0,
        }
    }

    pub fn update(&mut self, _game_state: &GameState, frame_time: f32) {
        // Update elapsed time
        self.elapsed_time += frame_time;

        // Store time-based performance metrics
        let frame_time_ms = frame_time * 1000.0;
        self.frame_times
            .push_back((self.elapsed_time, frame_time_ms));
        if self.frame_times.len() > 120 {
            self.frame_times.pop_front();
        }

        // Smooth FPS data to reduce noise
        let raw_fps = get_fps() as f32;
        self.fps_smoothing_buffer.push_back(raw_fps);
        if self.fps_smoothing_buffer.len() > 10 {
            self.fps_smoothing_buffer.pop_front();
        }

        // Calculate smoothed FPS
        let smoothed_fps = if !self.fps_smoothing_buffer.is_empty() {
            self.fps_smoothing_buffer.iter().sum::<f32>() / self.fps_smoothing_buffer.len() as f32
        } else {
            raw_fps
        };

        self.fps_history
            .push_back((self.elapsed_time, smoothed_fps));
        if self.fps_history.len() > 120 {
            self.fps_history.pop_front();
        }
    }

    pub fn render_ui(&mut self, ctx: &Context, game_state: &GameState) {
        // Main debug window
        Window::new("Debug Overlay")
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.toggle_value(&mut self.show_performance, "Performance");
                    ui.toggle_value(&mut self.show_server_state, "Server State");
                    ui.toggle_value(&mut self.show_mini_map, "Mini Map");
                    ui.toggle_value(&mut self.show_network, "Network");
                    ui.toggle_value(&mut self.show_rendering_toggles, "Rendering");
                });

                ui.separator();

                if self.show_performance {
                    self.render_performance_panel(ui);
                }

                if self.show_server_state {
                    self.render_server_state_panel(ui, game_state);
                }

                if self.show_mini_map {
                    self.render_mini_map_panel(ui, game_state);
                }

                if self.show_network {
                    self.render_network_panel(ui, game_state);
                }

                if self.show_rendering_toggles {
                    self.render_rendering_toggles_panel(ui);
                }
            });
    }

    fn render_performance_panel(&mut self, ui: &mut Ui) {
        ui.heading("Performance");
        ui.indent("performance_indent", |ui| {
            // FPS display
            let current_fps = self.fps_history.back().map(|(_, fps)| *fps).unwrap_or(0.0);
            ui.label(format!("FPS: {:.1}", current_fps));

            // Frame time display
            let current_frame_time = self
                .frame_times
                .back()
                .map(|(_, time)| *time)
                .unwrap_or(0.0);
            ui.label(format!("Frame Time: {:.2}ms", current_frame_time));

            // Performance statistics in two columns
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.strong("FPS Stats:");
                    if !self.fps_history.is_empty() {
                        let fps_values: Vec<f32> =
                            self.fps_history.iter().map(|(_, fps)| *fps).collect();
                        let min_fps = fps_values.iter().copied().fold(f32::INFINITY, f32::min);
                        let max_fps = fps_values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                        let avg_fps: f32 = fps_values.iter().sum::<f32>() / fps_values.len() as f32;

                        ui.label(format!("Min: {:.1}", min_fps));
                        ui.label(format!("Max: {:.1}", max_fps));
                        ui.label(format!("Avg: {:.1}", avg_fps));
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.strong("Frame Time (ms):");
                    if !self.frame_times.is_empty() {
                        let time_values: Vec<f32> =
                            self.frame_times.iter().map(|(_, time)| *time).collect();
                        let min_time = time_values.iter().copied().fold(f32::INFINITY, f32::min);
                        let max_time = time_values
                            .iter()
                            .copied()
                            .fold(f32::NEG_INFINITY, f32::max);
                        let avg_time: f32 =
                            time_values.iter().sum::<f32>() / time_values.len() as f32;

                        ui.label(format!("Min: {:.2}", min_time));
                        ui.label(format!("Max: {:.2}", max_time));
                        ui.label(format!("Avg: {:.2}", avg_time));
                    }
                });
            });

            ui.separator();

            // FPS graph using egui_plot (it works with macroquad!)
            if !self.fps_history.is_empty() {
                use egui_plot::{Line, Plot, PlotPoints};

                // Show last 30 seconds of data
                let time_window = 30.0;
                let x_max = self.elapsed_time;
                let x_min = (x_max - time_window).max(0.0);

                let plot = Plot::new("fps_plot")
                    .height(80.0)
                    .show_axes([true, true])
                    .show_grid([true, true])
                    .auto_bounds([false, true])
                    .include_x(x_min as f64)
                    .include_x(x_max as f64)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .x_axis_label("Time (s)")
                    .y_axis_label("FPS");

                plot.show(ui, |plot_ui| {
                    // Only show graph if we have multiple data points
                    if self.fps_history.len() > 1 {
                        let fps_points: PlotPoints = self
                            .fps_history
                            .iter()
                            .map(|(time, fps)| [*time as f64, *fps as f64])
                            .collect();

                        let line = Line::new(fps_points)
                            .name("FPS")
                            .color(egui::Color32::GREEN);
                        plot_ui.line(line);
                    } else {
                        // Show loading message when we don't have enough data
                        plot_ui.text(egui_plot::Text::new(
                            egui_plot::PlotPoint::new(0.5, 30.0),
                            "Collecting data...",
                        ));
                    }
                });
            }

            // Frame time graph
            if !self.frame_times.is_empty() {
                use egui_plot::{Line, Plot, PlotPoints};

                // Show last 30 seconds of data
                let time_window = 30.0;
                let x_max = self.elapsed_time;
                let x_min = (x_max - time_window).max(0.0);

                let plot = Plot::new("frame_time_plot")
                    .height(80.0)
                    .show_axes([true, true])
                    .show_grid([true, true])
                    .auto_bounds([false, true])
                    .include_x(x_min as f64)
                    .include_x(x_max as f64)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .x_axis_label("Time (s)")
                    .y_axis_label("Frame Time (ms)");

                plot.show(ui, |plot_ui| {
                    // Only show graph if we have multiple data points
                    if self.frame_times.len() > 1 {
                        let time_points: PlotPoints = self
                            .frame_times
                            .iter()
                            .map(|(time, frame_ms)| [*time as f64, *frame_ms as f64])
                            .collect();

                        let line = Line::new(time_points)
                            .name("Frame Time (ms)")
                            .color(egui::Color32::YELLOW);
                        plot_ui.line(line);
                    } else {
                        // Show loading message when we don't have enough data
                        plot_ui.text(egui_plot::Text::new(
                            egui_plot::PlotPoint::new(0.5, 16.0),
                            "Collecting data...",
                        ));
                    }
                });
            }
        });
    }

    fn render_server_state_panel(&mut self, ui: &mut Ui, game_state: &GameState) {
        ui.heading("Server State");
        ui.indent("server_state_indent", |ui| {
            // Team and Location info (moved from top-left overlay)
            ui.strong("Player Info:");

            let team_text = match game_state.player_team {
                Some(TeamId::Red) => "Team: RED",
                Some(TeamId::Blue) => "Team: BLUE",
                None => "Team: None",
            };
            ui.label(team_text);

            let location_text = match game_state.player_location {
                PlayerLocation::OutsideWorld(pos) => {
                    format!("Outside at ({:.1}, {:.1})", pos.x, pos.y)
                }
                PlayerLocation::InsideMech { floor, pos, .. } => {
                    format!(
                        "Inside Mech - Floor {} at ({:.1}, {:.1})",
                        floor + 1,
                        pos.x,
                        pos.y
                    )
                }
            };
            ui.label(location_text);

            if let Some(player_id) = game_state.player_id {
                ui.label(format!("Player ID: {}", player_id));
            }

            ui.separator();

            // Entity counts in two columns
            ui.strong("Entity Counts:");
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!("Players: {}", game_state.players.len()));
                    ui.label(format!("Mechs: {}", game_state.mechs.len()));
                    ui.label(format!("Stations: {}", game_state.stations.len()));
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.label(format!("Resources: {}", game_state.resources.len()));
                    ui.label(format!("Projectiles: {}", game_state.projectiles.len()));
                    ui.label(format!("Visible Tiles: {}", game_state.visible_tiles.len()));
                });
            });

            ui.separator();

            // ASCII tile view
            ui.label("ASCII World View:");
            ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                let ascii_view = self.generate_ascii_view(game_state);
                ui.add(
                    TextEdit::multiline(&mut ascii_view.as_str())
                        .font(FontId::monospace(12.0))
                        .desired_width(400.0),
                );
            });
        });
    }

    fn render_mini_map_panel(&mut self, ui: &mut Ui, _game_state: &GameState) {
        ui.heading("Mini Map");
        ui.indent("mini_map_indent", |ui| {
            ui.label("Mini map visualization coming soon...");
            ui.label("Will show overhead view of all game entities");

            ui.horizontal(|ui| {
                ui.label("Zoom:");
                ui.add(Slider::new(&mut self.mini_map_zoom, 0.1..=3.0));
            });
        });
    }

    fn render_network_panel(&mut self, ui: &mut Ui, _game_state: &GameState) {
        ui.heading("Network");
        ui.indent("network_indent", |ui| {
            ui.label("Connection Status: Connected"); // TODO: Get real status
            ui.label("Recent Messages:");

            ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                for msg in &self.message_history {
                    ui.label(msg);
                }
            });
        });
    }

    fn render_rendering_toggles_panel(&mut self, ui: &mut Ui) {
        ui.heading("Rendering Toggles");
        ui.indent("rendering_toggles_indent", |ui| {
            // Rendering toggles in two columns
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut self.render_tiles, "Render Tiles");
                    ui.checkbox(&mut self.render_mechs, "Render Mechs");
                    ui.checkbox(&mut self.render_players, "Render Players");
                    ui.checkbox(&mut self.render_stations, "Render Stations");
                    ui.checkbox(&mut self.render_resources, "Render Resources");
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.checkbox(&mut self.render_projectiles, "Render Projectiles");
                    ui.checkbox(&mut self.render_effects, "Render Effects");
                    ui.checkbox(&mut self.render_fog, "Render Fog of War");
                    ui.checkbox(&mut self.render_ui, "Render UI");
                });
            });

            ui.separator();

            if ui.button("Enable All").clicked() {
                self.render_tiles = true;
                self.render_mechs = true;
                self.render_players = true;
                self.render_stations = true;
                self.render_resources = true;
                self.render_projectiles = true;
                self.render_effects = true;
                self.render_fog = true;
                self.render_ui = true;
            }

            if ui.button("Disable All").clicked() {
                self.render_tiles = false;
                self.render_mechs = false;
                self.render_players = false;
                self.render_stations = false;
                self.render_resources = false;
                self.render_projectiles = false;
                self.render_effects = false;
                self.render_fog = false;
                self.render_ui = false;
            }
        });
    }

    fn generate_ascii_view(&self, game_state: &GameState) -> String {
        let (width, height) = self.ascii_grid_size;
        let mut grid = vec![vec![' '; width]; height];

        // Get player position for centering
        let player_pos = match game_state.player_location {
            PlayerLocation::OutsideWorld(world_pos) => world_pos.to_tile_pos(),
            PlayerLocation::InsideMech { pos, .. } => pos.to_tile_pos(),
        };

        let center_x = width / 2;
        let center_y = height / 2;

        // Fill grid with tiles
        for (tile_pos, tile_visual) in &game_state.visible_tiles {
            let rel_x = tile_pos.x - player_pos.x + center_x as i32;
            let rel_y = tile_pos.y - player_pos.y + center_y as i32;

            if rel_x >= 0 && rel_x < width as i32 && rel_y >= 0 && rel_y < height as i32 {
                grid[rel_y as usize][rel_x as usize] = self.tile_to_ascii(tile_visual);
            }
        }

        // Mark player position
        grid[center_y][center_x] = '@';

        // Mark other players
        for player in game_state.players.values() {
            let player_tile = match player.location {
                PlayerLocation::OutsideWorld(world_pos) => world_pos.to_tile_pos(),
                PlayerLocation::InsideMech { pos, .. } => pos.to_tile_pos(),
            };

            let rel_x = player_tile.x - player_pos.x + center_x as i32;
            let rel_y = player_tile.y - player_pos.y + center_y as i32;

            if rel_x >= 0 && rel_x < width as i32 && rel_y >= 0 && rel_y < height as i32 {
                let symbol = match player.team {
                    TeamId::Red => 'R',
                    TeamId::Blue => 'B',
                };
                grid[rel_y as usize][rel_x as usize] = symbol;
            }
        }

        // Mark mechs
        for mech in game_state.mechs.values() {
            let rel_x = mech.position.x - player_pos.x + center_x as i32;
            let rel_y = mech.position.y - player_pos.y + center_y as i32;

            if rel_x >= 0 && rel_x < width as i32 && rel_y >= 0 && rel_y < height as i32 {
                let symbol = match mech.team {
                    TeamId::Red => 'M',
                    TeamId::Blue => 'W', // W for mech (M is taken)
                };
                grid[rel_y as usize][rel_x as usize] = symbol;
            }
        }

        // Mark resources
        for resource in &game_state.resources {
            let rel_x = resource.position.x - player_pos.x + center_x as i32;
            let rel_y = resource.position.y - player_pos.y + center_y as i32;

            if rel_x >= 0 && rel_x < width as i32 && rel_y >= 0 && rel_y < height as i32 {
                grid[rel_y as usize][rel_x as usize] = '$';
            }
        }

        // Convert to string with line numbers for reference
        let mut result = String::new();
        for (i, row) in grid.iter().enumerate() {
            result.push_str(&format!("{:2}|", i));
            result.push_str(&row.iter().collect::<String>());
            result.push('\n');
        }

        // Add column numbers
        result.push_str("  +");
        for i in 0..width {
            if i % 10 == 0 {
                result.push_str(&format!("{}", i / 10));
            } else {
                result.push(' ');
            }
        }
        result.push('\n');
        result.push_str("  +");
        for i in 0..width {
            result.push_str(&format!("{}", i % 10));
        }
        result.push('\n');

        result
    }

    fn tile_to_ascii(&self, visual: &TileVisual) -> char {
        match visual {
            TileVisual::Floor { .. } => '.',
            TileVisual::Wall { .. } => '#',
            TileVisual::Window { .. } => 'w',
            TileVisual::Station {
                station_type,
                active,
            } => {
                match station_type {
                    StationType::Pilot => {
                        if *active {
                            'P'
                        } else {
                            'p'
                        }
                    }
                    StationType::WeaponLaser => {
                        if *active {
                            'L'
                        } else {
                            'l'
                        }
                    }
                    StationType::WeaponProjectile => {
                        if *active {
                            'T'
                        } else {
                            't'
                        }
                    }
                    StationType::Shield => {
                        if *active {
                            'S'
                        } else {
                            's'
                        }
                    }
                    StationType::Engine => {
                        if *active {
                            'E'
                        } else {
                            'e'
                        }
                    }
                    StationType::Repair => {
                        if *active {
                            'H'
                        } else {
                            'h'
                        }
                    }
                    StationType::Upgrade => {
                        if *active {
                            'U'
                        } else {
                            'u'
                        }
                    }
                    StationType::Electrical => {
                        if *active {
                            'C'
                        } else {
                            'c'
                        }
                    } // 'C' for Circuit
                }
            }
            TileVisual::Turret { firing, .. } => {
                if *firing {
                    'X'
                } else {
                    'x'
                }
            }
            TileVisual::TransitionFade { .. } => '~',
        }
    }

    pub fn log_server_message(&mut self, message: &str) {
        self.message_counter += 1;
        self.message_history
            .push_back(format!("[{}] {}", self.message_counter, message));
        if self.message_history.len() > 20 {
            self.message_history.pop_front();
        }
    }
}

// No-op implementation for release builds
#[cfg(not(debug_assertions))]
pub struct DebugOverlay {
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

#[cfg(not(debug_assertions))]
impl DebugOverlay {
    pub fn new() -> Self {
        Self {
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

    pub fn update(&mut self, _game_state: &crate::game_state::GameState, _frame_time: f32) {}
    pub fn render_ui(&mut self, _ctx: &egui::Context, _game_state: &crate::game_state::GameState) {}
    pub fn log_server_message(&mut self, _message: &str) {}
}
