use macroquad::prelude::*;
use shared::{types::*, constants::*};
use crate::game_state::*;
use uuid::Uuid;

pub struct Renderer {
    // Could store textures and other rendering resources here
}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, game_state: &GameState) {
        // Apply camera transform
        let cam_x = -game_state.camera_offset.0;
        let cam_y = -game_state.camera_offset.1;

        match game_state.player_location {
            PlayerLocation::OutsideWorld(_) => {
                self.render_world_view(game_state, cam_x, cam_y);
            }
            PlayerLocation::InsideMech { mech_id, floor, .. } => {
                if let Some(mech) = game_state.mechs.get(&mech_id) {
                    self.render_mech_interior(mech, floor, cam_x, cam_y);
                    self.render_stations_on_floor(game_state, mech_id, floor);
                    self.render_players_on_floor(game_state, mech_id, floor, cam_x, cam_y);
                }
            }
        }

        // Render UI overlay
        self.render_ui(game_state);
    }

    fn render_world_view(&self, game_state: &GameState, cam_x: f32, cam_y: f32) {
        // Draw arena boundaries
        let arena_width = ARENA_WIDTH_TILES as f32 * TILE_SIZE;
        let arena_height = ARENA_HEIGHT_TILES as f32 * TILE_SIZE;
        
        draw_rectangle_lines(
            cam_x,
            cam_y,
            arena_width,
            arena_height,
            3.0,
            GRAY
        );

        // Draw mechs as large rectangles
        for mech in game_state.mechs.values() {
            let mech_size = MECH_SIZE_TILES as f32 * TILE_SIZE;
            let color = match mech.team {
                TeamId::Red => Color::new(0.8, 0.2, 0.2, 1.0),
                TeamId::Blue => Color::new(0.2, 0.2, 0.8, 1.0),
            };
            
            draw_rectangle(
                cam_x + mech.position.x as f32 * TILE_SIZE,
                cam_y + mech.position.y as f32 * TILE_SIZE,
                mech_size,
                mech_size,
                color
            );
            
            // Draw mech outline
            draw_rectangle_lines(
                cam_x + mech.position.x as f32 * TILE_SIZE,
                cam_y + mech.position.y as f32 * TILE_SIZE,
                mech_size,
                mech_size,
                2.0,
                WHITE
            );
        }

        // Draw resources
        for resource in &game_state.resources {
            let color = match resource.resource_type {
                ResourceType::ScrapMetal => DARKGRAY,
                ResourceType::ComputerComponents => GREEN,
                ResourceType::Wiring => YELLOW,
                ResourceType::Batteries => ORANGE,
            };
            
            draw_circle(
                cam_x + resource.position.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                cam_y + resource.position.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                TILE_SIZE / 3.0,
                color
            );
        }

        // Draw projectiles
        for projectile in &game_state.projectiles {
            draw_circle(
                cam_x + projectile.position.x,
                cam_y + projectile.position.y,
                5.0,
                YELLOW
            );
        }

        // Draw weapon effects
        for effect in &game_state.weapon_effects {
            if effect.weapon_type == StationType::WeaponLaser {
                // Draw laser beam
                if let Some(mech) = game_state.mechs.get(&effect.mech_id) {
                    let start_x = cam_x + mech.position.x as f32 * TILE_SIZE + (MECH_SIZE_TILES as f32 * TILE_SIZE) / 2.0;
                    let start_y = cam_y + mech.position.y as f32 * TILE_SIZE + (MECH_SIZE_TILES as f32 * TILE_SIZE) / 2.0;
                    let end_x = cam_x + effect.target.x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
                    let end_y = cam_y + effect.target.y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
                    
                    draw_line(
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        3.0 * effect.timer,
                        Color::new(1.0, 0.0, 0.0, effect.timer)
                    );
                }
            }
        }

        // Draw oxygen tether lines for players carrying resources
        for player in game_state.players.values() {
            if let PlayerLocation::OutsideWorld(pos) = player.location {
                if player.carrying_resource.is_some() {
                    // Find the nearest team mech
                    let nearest_mech = game_state.mechs.values()
                        .filter(|m| m.team == player.team)
                        .min_by(|a, b| {
                            let mech_center_a = WorldPos::new(
                                (a.position.x as f32 + MECH_SIZE_TILES as f32 / 2.0) * TILE_SIZE,
                                (a.position.y as f32 + MECH_SIZE_TILES as f32 / 2.0) * TILE_SIZE
                            );
                            let mech_center_b = WorldPos::new(
                                (b.position.x as f32 + MECH_SIZE_TILES as f32 / 2.0) * TILE_SIZE,
                                (b.position.y as f32 + MECH_SIZE_TILES as f32 / 2.0) * TILE_SIZE
                            );
                            let dist_a = pos.distance_to(mech_center_a);
                            let dist_b = pos.distance_to(mech_center_b);
                            dist_a.partial_cmp(&dist_b).unwrap()
                        });
                    
                    if let Some(mech) = nearest_mech {
                        let mech_center = WorldPos::new(
                            (mech.position.x as f32 + MECH_SIZE_TILES as f32 / 2.0) * TILE_SIZE,
                            (mech.position.y as f32 + MECH_SIZE_TILES as f32 / 2.0) * TILE_SIZE
                        );
                        let distance = pos.distance_to(mech_center) / TILE_SIZE;
                        
                        // Color based on distance (green to red)
                        let ratio = (distance / MAX_DISTANCE_FROM_MECH).clamp(0.0, 1.0);
                        let mut tether_color = Color::new(ratio, 1.0 - ratio, 0.0, 0.7);
                        let mut line_width = 3.0;
                        
                        // Flash red and thicken line when at limit
                        if distance >= MAX_DISTANCE_FROM_MECH {
                            tether_color = Color::new(1.0, 0.0, 0.0, 0.9);
                            line_width = 5.0;
                        }
                        
                        // Draw tether line
                        draw_line(
                            cam_x + mech_center.x,
                            cam_y + mech_center.y,
                            cam_x + pos.x,
                            cam_y + pos.y,
                            line_width,
                            tether_color
                        );
                    }
                }
            }
        }

        // Draw players
        for player in game_state.players.values() {
            if let PlayerLocation::OutsideWorld(pos) = player.location {
                let color = match player.team {
                    TeamId::Red => Color::new(1.0, 0.3, 0.3, 1.0),
                    TeamId::Blue => Color::new(0.3, 0.3, 1.0, 1.0),
                };
                
                draw_circle(
                    cam_x + pos.x,
                    cam_y + pos.y,
                    TILE_SIZE / 2.0,
                    color
                );
                
                // Draw player name
                draw_text(
                    &player.name,
                    cam_x + pos.x - 20.0,
                    cam_y + pos.y - TILE_SIZE - 5.0,
                    16.0,
                    WHITE
                );
                
                // Draw resource being carried
                if let Some(resource_type) = player.carrying_resource {
                    let resource_color = match resource_type {
                        ResourceType::ScrapMetal => DARKGRAY,
                        ResourceType::ComputerComponents => GREEN,
                        ResourceType::Wiring => YELLOW,
                        ResourceType::Batteries => ORANGE,
                    };
                    draw_circle(
                        cam_x + pos.x + TILE_SIZE,
                        cam_y + pos.y,
                        TILE_SIZE / 4.0,
                        resource_color
                    );
                }
            }
        }
    }

    fn render_mech_interior(&self, mech: &MechState, current_floor: u8, cam_x: f32, cam_y: f32) {
        let floor = &mech.floors[current_floor as usize];
        
        // Draw floor tiles
        for (y, row) in floor.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let tile_x = cam_x + x as f32 * TILE_SIZE;
                let tile_y = cam_y + y as f32 * TILE_SIZE;
                
                match tile {
                    TileType::Empty => {} // Don't draw anything
                    TileType::Floor => {
                        draw_rectangle(tile_x, tile_y, TILE_SIZE, TILE_SIZE, DARKGRAY);
                        draw_rectangle_lines(tile_x, tile_y, TILE_SIZE, TILE_SIZE, 1.0, GRAY);
                    }
                    TileType::Wall => {
                        draw_rectangle(tile_x, tile_y, TILE_SIZE, TILE_SIZE, LIGHTGRAY);
                    }
                    TileType::Station(station_type) => {
                        draw_rectangle(tile_x, tile_y, TILE_SIZE, TILE_SIZE, DARKGRAY);
                        let color = self.get_station_color(*station_type);
                        draw_rectangle(
                            tile_x + TILE_SIZE * 0.1,
                            tile_y + TILE_SIZE * 0.1,
                            TILE_SIZE * 0.8,
                            TILE_SIZE * 0.8,
                            color
                        );
                    }
                    TileType::Ladder => {
                        draw_rectangle(tile_x, tile_y, TILE_SIZE, TILE_SIZE, DARKGRAY);
                        draw_text("↕", tile_x + 5.0, tile_y + TILE_SIZE - 5.0, 30.0, YELLOW);
                    }
                }
            }
        }
    }

    fn render_stations_on_floor(&self, game_state: &GameState, mech_id: Uuid, floor: u8) {
        // Draw station labels
        for station in game_state.stations.values() {
            if station.mech_id == mech_id && station.floor == floor {
                let x = station.position.x as f32 * TILE_SIZE;
                let y = station.position.y as f32 * TILE_SIZE - 5.0;
                
                let label = match station.station_type {
                    StationType::WeaponLaser => "LASER",
                    StationType::WeaponProjectile => "GUN",
                    StationType::Engine => "ENGINE",
                    StationType::Shield => "SHIELD",
                    StationType::Repair => "REPAIR",
                    StationType::Electrical => "ELEC",
                    StationType::Upgrade => "UPGRADE",
                };
                
                draw_text(label, x, y, 16.0, WHITE);
            }
        }
    }

    fn render_players_on_floor(&self, game_state: &GameState, mech_id: Uuid, floor: u8, cam_x: f32, cam_y: f32) {
        // Draw players on this floor of the mech
        for player in game_state.players.values() {
            if let PlayerLocation::InsideMech { mech_id: player_mech_id, floor: player_floor, pos } = player.location {
                if player_mech_id == mech_id && player_floor == floor {
                    let color = match player.team {
                        TeamId::Red => Color::new(1.0, 0.3, 0.3, 1.0),
                        TeamId::Blue => Color::new(0.3, 0.3, 1.0, 1.0),
                    };
                    
                    draw_circle(
                        cam_x + pos.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                        cam_y + pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                        TILE_SIZE / 2.5,
                        color
                    );
                    
                    // Draw player name
                    draw_text(
                        &player.name,
                        cam_x + pos.x as f32 * TILE_SIZE - 20.0,
                        cam_y + pos.y as f32 * TILE_SIZE - 5.0,
                        14.0,
                        WHITE
                    );
                }
            }
        }
    }

    fn get_station_color(&self, station_type: StationType) -> Color {
        match station_type {
            StationType::WeaponLaser => RED,
            StationType::WeaponProjectile => ORANGE,
            StationType::Engine => BLUE,
            StationType::Shield => SKYBLUE,
            StationType::Repair => GREEN,
            StationType::Electrical => YELLOW,
            StationType::Upgrade => PURPLE,
        }
    }

    fn render_ui(&self, game_state: &GameState) {
        // Draw team and location info
        let team_text = match game_state.player_team {
            Some(TeamId::Red) => "Team: RED",
            Some(TeamId::Blue) => "Team: BLUE",
            None => "Team: None",
        };
        draw_text(team_text, 10.0, 30.0, 20.0, WHITE);
        
        let location_text = match game_state.player_location {
            PlayerLocation::OutsideWorld(pos) => format!("Outside at ({}, {})", pos.x, pos.y),
            PlayerLocation::InsideMech { floor, pos, .. } => {
                format!("Inside Mech - Floor {} at ({}, {})", floor + 1, pos.x, pos.y)
            }
        };
        
        draw_text(&location_text, 10.0, 50.0, 20.0, WHITE);
        
        // Draw mech health bars
        let mut y_offset = 80.0;
        for mech in game_state.mechs.values() {
            let team_color = match mech.team {
                TeamId::Red => RED,
                TeamId::Blue => BLUE,
            };
            
            draw_text(&format!("{:?} Mech", mech.team), 10.0, y_offset, 18.0, team_color);
            
            // Health bar
            draw_rectangle(10.0, y_offset + 5.0, 200.0, 10.0, DARKGRAY);
            draw_rectangle(10.0, y_offset + 5.0, 200.0 * (mech.health as f32 / 100.0), 10.0, GREEN);
            
            // Shield bar
            draw_rectangle(10.0, y_offset + 17.0, 200.0, 10.0, DARKGRAY);
            draw_rectangle(10.0, y_offset + 17.0, 200.0 * (mech.shield as f32 / 50.0), 10.0, SKYBLUE);
            
            y_offset += 40.0;
        }
        
        // Draw controls hint
        draw_text("WASD: Move | Space: Action | Q: Exit Mech", 10.0, screen_height() - 20.0, 16.0, WHITE);
        
        if let PlayerLocation::InsideMech { floor, .. } = game_state.player_location {
            draw_text(
                &format!("Current Floor: {} | Up/Down arrows at ladders to change floors", floor + 1),
                10.0,
                screen_height() - 40.0,
                16.0,
                WHITE
            );
            
            // Show station controls if at a station
            for station in game_state.stations.values() {
                if let Some(player_id) = game_state.player_id {
                    if station.operated_by == Some(player_id) {
                        draw_text(
                            "Station Controls: Press 1-5 to operate",
                            10.0,
                            screen_height() - 60.0,
                            16.0,
                            YELLOW
                        );
                        break;
                    }
                }
            }
        }
    }
}