use crate::game_state::*;
use macroquad::prelude::*;
use shared::types::*;

pub fn render_ui(game_state: &GameState) {
    // Team and location info moved to debug overlay to avoid overlap
    // render_team_and_location_info(game_state);
    render_mech_status_bars(game_state);
    render_control_hints(game_state);
}

fn render_team_and_location_info(game_state: &GameState) {
    // Team info
    let team_text = match game_state.player_team {
        Some(TeamId::Red) => "Team: RED",
        Some(TeamId::Blue) => "Team: BLUE",
        None => "Team: None",
    };
    draw_text(team_text, 10.0, 30.0, 20.0, WHITE);

    // Location info
    let location_text = match game_state.player_location {
        PlayerLocation::OutsideWorld(pos) => format!("Outside at ({}, {})", pos.x, pos.y),
        PlayerLocation::InsideMech { pos, .. } => {
            let floor = pos.floor();
            let tile_pos = pos.tile_pos();
            format!(
                "Inside Mech - Floor {} at ({}, {})",
                floor + 1,
                tile_pos.x,
                tile_pos.y
            )
        }
    };
    draw_text(&location_text, 10.0, 50.0, 20.0, WHITE);
}

fn render_mech_status_bars(game_state: &GameState) {
    let mut y_offset = 80.0;

    for mech in game_state.mechs.values() {
        let team_color = match mech.team {
            TeamId::Red => RED,
            TeamId::Blue => BLUE,
        };

        draw_text(
            &format!("{:?} Mech", mech.team),
            10.0,
            y_offset,
            18.0,
            team_color,
        );

        // Health bar
        render_status_bar(
            10.0,
            y_offset + 5.0,
            200.0,
            10.0,
            mech.health as f32 / 100.0,
            GREEN,
        );

        // Shield bar
        render_status_bar(
            10.0,
            y_offset + 17.0,
            200.0,
            10.0,
            mech.shield as f32 / 50.0,
            SKYBLUE,
        );

        y_offset += 40.0;
    }
}

fn render_status_bar(x: f32, y: f32, width: f32, height: f32, fill_ratio: f32, color: Color) {
    draw_rectangle(x, y, width, height, DARKGRAY);
    draw_rectangle(x, y, width * fill_ratio, height, color);
}

fn render_control_hints(game_state: &GameState) {
    // Basic controls
    draw_text(
        "WASD: Move | Space: Action | Q: Exit Mech",
        10.0,
        screen_height() - 20.0,
        16.0,
        WHITE,
    );

    // Context-specific hints
    if let PlayerLocation::InsideMech { pos, .. } = game_state.player_location {
        let floor = pos.floor();
        draw_text(
            &format!(
                "Current Floor: {} | Up/Down arrows at ladders to change floors",
                floor + 1
            ),
            10.0,
            screen_height() - 40.0,
            16.0,
            WHITE,
        );

        // Station controls hint
        if is_player_at_station(game_state) {
            draw_text(
                "Station Controls: Press 1-5 to operate",
                10.0,
                screen_height() - 60.0,
                16.0,
                YELLOW,
            );
        }
    }
}

fn is_player_at_station(game_state: &GameState) -> bool {
    if let Some(player_id) = game_state.player_id {
        game_state
            .stations
            .values()
            .any(|station| station.operated_by == Some(player_id))
    } else {
        false
    }
}
