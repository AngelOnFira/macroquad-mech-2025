use super::utils::*;
use crate::game_state::*;
use crate::vision::{ClientVisionSystem, FogOfWarRenderer};
use macroquad::prelude::*;
use shared::{constants::*, coordinates::MechDoorPositions, types::*};

#[cfg(feature = "profiling")]
use profiling::scope;


pub fn render_world_view(game_state: &GameState, cam_x: f32, cam_y: f32) {
    render_world_view_with_vision(game_state, cam_x, cam_y, None);
}

pub fn render_world_view_with_vision(
    game_state: &GameState,
    cam_x: f32,
    cam_y: f32,
    vision_system: Option<&ClientVisionSystem>,
) {
    {
        #[cfg(feature = "profiling")]
        scope!("grass_background");
        render_grass_background(cam_x, cam_y, vision_system);
    }
    {
        #[cfg(feature = "profiling")]
        scope!("arena_boundaries");
        render_arena_boundaries(cam_x, cam_y);
    }
    {
        #[cfg(feature = "profiling")]
        scope!("mechs");
        render_mechs(game_state, cam_x, cam_y, vision_system);
    }
    {
        #[cfg(feature = "profiling")]
        scope!("world_tiles");
        render_world_tiles(game_state, cam_x, cam_y, vision_system);
    }
    {
        #[cfg(feature = "profiling")]
        scope!("resources");
        render_resources(game_state, cam_x, cam_y, vision_system);
    }
    {
        #[cfg(feature = "profiling")]
        scope!("projectiles");
        render_projectiles(game_state, cam_x, cam_y, vision_system);
    }
    {
        #[cfg(feature = "profiling")]
        scope!("players");
        render_players_in_world(game_state, cam_x, cam_y, vision_system);
    }

    // Render fog overlay for completely invisible areas
    {
        #[cfg(feature = "profiling")]
        scope!("fog_overlay");
        if let Some(vision) = vision_system {
            render_fog_overlay(vision, cam_x, cam_y);
        }
    }

    // Debug info
    {
        #[cfg(feature = "profiling")]
        scope!("debug_text");
        draw_text(
            &format!("Camera: ({:.1}, {:.1})", -cam_x, -cam_y),
            10.0,
            30.0,
            20.0,
            WHITE,
        );
        draw_text(
            &format!(
                "Mechs: {}, Players: {}",
                game_state.mechs.len(),
                game_state.players.len()
            ),
            10.0,
            50.0,
            20.0,
            WHITE,
        );
        if let Some(player_id) = game_state.player_id {
            if let PlayerLocation::OutsideWorld(pos) = game_state.player_location {
                draw_text(
                    &format!("Player pos: ({:.1}, {:.1})", pos.x, pos.y),
                    10.0,
                    70.0,
                    20.0,
                    WHITE,
                );
            }
        }
    }
}

fn render_grass_background(cam_x: f32, cam_y: f32, vision_system: Option<&ClientVisionSystem>) {
    let grass_color = Color::new(0.2, 0.6, 0.2, 1.0);
    let grass_tile_size = TILE_SIZE * 2.0;

    // Calculate visible area with some padding
    let screen_w = screen_width();
    let screen_h = screen_height();
    let start_x = ((-cam_x / grass_tile_size).floor()) as i32 - 1;
    let start_y = ((-cam_y / grass_tile_size).floor()) as i32 - 1;
    let end_x = ((-cam_x + screen_w) / grass_tile_size).ceil() as i32 + 1;
    let end_y = ((-cam_y + screen_h) / grass_tile_size).ceil() as i32 + 1;

    for ty in start_y..end_y {
        for tx in start_x..end_x {
            let x = tx as f32 * grass_tile_size;
            let y = ty as f32 * grass_tile_size;

            // Vary grass color slightly for texture
            let variation = ((tx * 17 + ty * 13) % 20) as f32 / 200.0 - 0.05;
            let mut tile_color = Color::new(
                (grass_color.r + variation).clamp(0.0, 1.0),
                (grass_color.g + variation).clamp(0.0, 1.0),
                (grass_color.b + variation).clamp(0.0, 1.0),
                1.0,
            );

            // Apply fog of war if vision system is available
            if let Some(vision) = vision_system {
                let tile_pos = TilePos::new((x / TILE_SIZE) as i32, (y / TILE_SIZE) as i32);
                let visibility = vision.get_visibility(tile_pos);
                tile_color = FogOfWarRenderer::apply_fog_to_color(tile_color, visibility);
            }

            draw_rectangle(
                cam_x + x,
                cam_y + y,
                grass_tile_size,
                grass_tile_size,
                tile_color,
            );
        }
    }
}

fn render_arena_boundaries(cam_x: f32, cam_y: f32) {
    let arena_width = ARENA_WIDTH_TILES as f32 * TILE_SIZE;
    let arena_height = ARENA_HEIGHT_TILES as f32 * TILE_SIZE;

    draw_rectangle_lines(cam_x, cam_y, arena_width, arena_height, 3.0, GRAY);
}

fn render_mechs(
    game_state: &GameState,
    cam_x: f32,
    cam_y: f32,
    vision_system: Option<&ClientVisionSystem>,
) {
    for mech in game_state.mechs.values() {
        let mech_size = MECH_SIZE_TILES as f32 * TILE_SIZE;
        let mut color = get_team_color(mech.team);
        let mut outline_color = WHITE;

        let mech_world = mech.position.to_world();
        let mech_x = cam_x + mech_world.x;
        let mech_y = cam_y + mech_world.y;

        // Apply fog of war to mech based on its position
        if let Some(vision) = vision_system {
            let visibility = vision.get_visibility(mech.position);
            color = FogOfWarRenderer::apply_fog_to_color(color, visibility);
            outline_color = FogOfWarRenderer::apply_fog_to_color(outline_color, visibility);
        }

        // Main body
        draw_rectangle(mech_x, mech_y, mech_size, mech_size, color);
        draw_rectangle_lines(mech_x, mech_y, mech_size, mech_size, 2.0, outline_color);

        // Render visible interior tiles if looking into mech
        if let Some(vision) = vision_system {
            let interior_tiles = vision.get_visible_interior_for_mech(mech.id);
            for (floor, interior_pos, visibility) in interior_tiles {
                if visibility > 0.1 {
                    render_visible_interior_tile(
                        mech,
                        floor,
                        interior_pos,
                        visibility,
                        cam_x,
                        cam_y,
                    );
                }
            }
        }
    }
}

fn render_visible_interior_tile(
    mech: &MechState,
    floor: u8,
    interior_pos: TilePos,
    visibility: f32,
    cam_x: f32,
    cam_y: f32,
) {
    use shared::MechInteriorCoordinates;

    // Convert interior position to world position for rendering
    let world_pos = MechInteriorCoordinates::interior_to_world(mech.position, floor, interior_pos);
    let world_coords = world_pos.to_world();
    let tile_x = cam_x + world_coords.x;
    let tile_y = cam_y + world_coords.y;

    // Render as a translucent floor tile that can be seen through windows/doors
    let mut interior_color = Color::new(0.3, 0.3, 0.4, visibility * 0.7);
    interior_color = FogOfWarRenderer::apply_fog_to_color(interior_color, visibility);

    draw_rectangle(tile_x, tile_y, TILE_SIZE, TILE_SIZE, interior_color);

    // Add a subtle grid to show it's interior space
    let grid_color = Color::new(0.5, 0.5, 0.6, visibility * 0.3);
    draw_rectangle_lines(tile_x, tile_y, TILE_SIZE, TILE_SIZE, 1.0, grid_color);
}

fn render_world_tiles(
    game_state: &GameState,
    cam_x: f32,
    cam_y: f32,
    vision_system: Option<&ClientVisionSystem>,
) {
    // Render visible tiles sent from server
    for (tile_pos, tile_visual) in &game_state.visible_tiles {
        let tile_world = tile_pos.to_world();
        let tile_x = cam_x + tile_world.x;
        let tile_y = cam_y + tile_world.y;

        // Apply fog of war if vision system is available
        if let Some(vision) = vision_system {
            let visibility = vision.get_visibility(*tile_pos);
            if visibility > 0.05 {
                // Only render if somewhat visible
                super::hybrid_tiles::render_tile_visual_with_visibility(
                    tile_visual,
                    tile_x,
                    tile_y,
                    TILE_SIZE,
                    visibility,
                );
            }
        } else {
            super::hybrid_tiles::render_tile_visual(tile_visual, tile_x, tile_y, TILE_SIZE);
        }
    }

    // Still render mech-related tiles for now as fallback
    // TODO: Remove once server sends all tiles
    if game_state.visible_tiles.is_empty() {
        for mech in game_state.mechs.values() {
            let team_color = get_team_color(mech.team);

            // Render door tiles using door position abstraction
            let doors = MechDoorPositions::from_mech_position(mech.position);
            render_door_tile(
                doors.left_door.x,
                doors.left_door.y,
                team_color,
                cam_x,
                cam_y,
            );
            render_door_tile(
                doors.right_door.x,
                doors.right_door.y,
                team_color,
                cam_x,
                cam_y,
            );
        }
    }
}

fn render_door_tile(x: i32, y: i32, team_color: Color, cam_x: f32, cam_y: f32) {
    let tile_x = cam_x + x as f32 * TILE_SIZE;
    let tile_y = cam_y + y as f32 * TILE_SIZE;

    // Door background (darker than mech)
    draw_rectangle(
        tile_x,
        tile_y,
        TILE_SIZE,
        TILE_SIZE,
        Color::new(
            team_color.r * 0.3,
            team_color.g * 0.3,
            team_color.b * 0.3,
            1.0,
        ),
    );

    // Door outline
    draw_rectangle_lines(tile_x, tile_y, TILE_SIZE, TILE_SIZE, 2.0, WHITE);

    // Visual entry indicator - just a subtle arrow or pattern
    let arrow_color = Color::new(1.0, 1.0, 1.0, 0.5);
    draw_line(
        tile_x + TILE_SIZE / 2.0,
        tile_y + TILE_SIZE * 0.3,
        tile_x + TILE_SIZE / 2.0,
        tile_y + TILE_SIZE * 0.7,
        2.0,
        arrow_color,
    );
    draw_line(
        tile_x + TILE_SIZE * 0.3,
        tile_y + TILE_SIZE * 0.5,
        tile_x + TILE_SIZE / 2.0,
        tile_y + TILE_SIZE * 0.7,
        2.0,
        arrow_color,
    );
    draw_line(
        tile_x + TILE_SIZE * 0.7,
        tile_y + TILE_SIZE * 0.5,
        tile_x + TILE_SIZE / 2.0,
        tile_y + TILE_SIZE * 0.7,
        2.0,
        arrow_color,
    );
}

fn render_resources(
    game_state: &GameState,
    cam_x: f32,
    cam_y: f32,
    vision_system: Option<&ClientVisionSystem>,
) {
    for resource in &game_state.resources {
        let mut color = get_resource_color(resource.resource_type);

        // Apply fog of war if vision system is available
        if let Some(vision) = vision_system {
            let tile_pos = TilePos::new(resource.position.x, resource.position.y);
            let visibility = vision.get_visibility(tile_pos);
            if visibility < 0.05 {
                continue; // Don't render invisible resources
            }
            color = FogOfWarRenderer::apply_fog_to_color(color, visibility);
        }

        draw_circle(
            cam_x + resource.position.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            cam_y + resource.position.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            TILE_SIZE / 3.0,
            color,
        );
    }
}

fn render_projectiles(
    game_state: &GameState,
    cam_x: f32,
    cam_y: f32,
    vision_system: Option<&ClientVisionSystem>,
) {
    for projectile in &game_state.projectiles {
        let mut color = YELLOW;

        // Apply fog of war if vision system is available
        if let Some(vision) = vision_system {
            let tile_pos = WorldPos::new(projectile.position.x, projectile.position.y).to_tile();
            let visibility = vision.get_visibility(tile_pos);
            if visibility < 0.05 {
                continue; // Don't render invisible projectiles
            }
            color = FogOfWarRenderer::apply_fog_to_color(color, visibility);
        }

        draw_circle(
            cam_x + projectile.position.x,
            cam_y + projectile.position.y,
            5.0,
            color,
        );
    }
}

fn render_players_in_world(
    game_state: &GameState,
    cam_x: f32,
    cam_y: f32,
    vision_system: Option<&ClientVisionSystem>,
) {
    for player in game_state.players.values() {
        if let PlayerLocation::OutsideWorld(pos) = player.location {
            let mut color = get_player_color(player.team);
            let mut text_color = WHITE;

            // Apply fog of war if vision system is available
            if let Some(vision) = vision_system {
                let tile_pos = pos.to_tile();
                let visibility = vision.get_visibility(tile_pos);
                if visibility < 0.05 {
                    continue; // Don't render invisible players
                }
                color = FogOfWarRenderer::apply_fog_to_color(color, visibility);
                text_color = FogOfWarRenderer::apply_fog_to_color(text_color, visibility);
            }

            // Player body
            draw_circle(cam_x + pos.x, cam_y + pos.y, TILE_SIZE / 2.0, color);

            // Player name
            draw_text(
                &player.name,
                cam_x + pos.x - 20.0,
                cam_y + pos.y - TILE_SIZE - 5.0,
                16.0,
                text_color,
            );

            // Resource being carried
            if let Some(resource_type) = player.carrying_resource {
                let mut resource_color = get_resource_color(resource_type);
                if let Some(vision) = vision_system {
                    let tile_pos = pos.to_tile();
                    let visibility = vision.get_visibility(tile_pos);
                    resource_color =
                        FogOfWarRenderer::apply_fog_to_color(resource_color, visibility);
                }
                draw_circle(
                    cam_x + pos.x + TILE_SIZE,
                    cam_y + pos.y,
                    TILE_SIZE / 4.0,
                    resource_color,
                );
            }
        }
    }
}

fn render_fog_overlay(vision_system: &ClientVisionSystem, cam_x: f32, cam_y: f32) {
    // Calculate visible area
    let screen_w = screen_width();
    let screen_h = screen_height();
    let start_x = ((-cam_x / TILE_SIZE).floor()) as i32 - 2;
    let start_y = ((-cam_y / TILE_SIZE).floor()) as i32 - 2;
    let end_x = ((-cam_x + screen_w) / TILE_SIZE).ceil() as i32 + 2;
    let end_y = ((-cam_y + screen_h) / TILE_SIZE).ceil() as i32 + 2;

    // Render fog overlay for invisible tiles
    for ty in start_y..end_y {
        for tx in start_x..end_x {
            let tile_pos = TilePos::new(tx, ty);

            if !vision_system.is_visible(tile_pos) {
                let tile_x = cam_x + tx as f32 * TILE_SIZE;
                let tile_y = cam_y + ty as f32 * TILE_SIZE;

                // Use edge fade for smooth fog transitions
                let edge_fade = FogOfWarRenderer::calculate_edge_fade(tile_pos, vision_system, 3);
                if edge_fade > 0.0 {
                    let fog_alpha = (1.0 - edge_fade) * 0.8; // Max 80% opacity
                    let fog_color = Color::new(0.0, 0.0, 0.0, fog_alpha);

                    draw_rectangle(tile_x, tile_y, TILE_SIZE, TILE_SIZE, fog_color);
                } else {
                    // Completely fogged area
                    draw_rectangle(
                        tile_x,
                        tile_y,
                        TILE_SIZE,
                        TILE_SIZE,
                        FogOfWarRenderer::get_fog_overlay_color(),
                    );
                }
            }
        }
    }
}
