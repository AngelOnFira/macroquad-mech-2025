use macroquad::prelude::*;
use shared::types::Direction;

pub struct InputHandler {
    last_action_pressed: bool,
}

pub struct InputState {
    pub direction: Option<Direction>,
    pub action_pressed: bool,
    pub exit_mech_pressed: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            last_action_pressed: false,
        }
    }

    pub fn update(&mut self) -> InputState {
        let mut state = InputState {
            direction: None,
            action_pressed: false,
            exit_mech_pressed: false,
        };

        // Movement
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            state.direction = Some(Direction::Up);
        } else if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            state.direction = Some(Direction::Down);
        } else if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            state.direction = Some(Direction::Left);
        } else if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            state.direction = Some(Direction::Right);
        }

        // Action key (Space) - detect press, not hold
        let action_down = is_key_down(KeyCode::Space);
        state.action_pressed = action_down && !self.last_action_pressed;
        self.last_action_pressed = action_down;

        // Exit mech key
        state.exit_mech_pressed = is_key_pressed(KeyCode::Q);

        state
    }
}

impl InputState {
    pub fn has_input(&self) -> bool {
        self.direction.is_some() || self.action_pressed
    }
}