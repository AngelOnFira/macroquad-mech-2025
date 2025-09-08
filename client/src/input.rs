use macroquad::prelude::*;

pub struct InputHandler {
    last_action_pressed: bool,
}

pub struct InputState {
    pub movement: (f32, f32), // x, y velocity (-1.0 to 1.0)
    pub action_pressed: bool,
    pub exit_mech_pressed: bool,
    pub floor_transition_pressed: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            last_action_pressed: false,
        }
    }

    pub fn update(&mut self) -> InputState {
        let mut state = InputState {
            movement: (0.0, 0.0),
            action_pressed: false,
            exit_mech_pressed: false,
            floor_transition_pressed: false,
        };

        // Movement - combine multiple directions for diagonal movement
        let mut movement_x = 0.0;
        let mut movement_y = 0.0;

        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            movement_y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            movement_y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            movement_x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            movement_x += 1.0;
        }

        // Normalize diagonal movement
        if movement_x != 0.0 || movement_y != 0.0 {
            let magnitude = ((movement_x * movement_x + movement_y * movement_y) as f32).sqrt();
            movement_x /= magnitude;
            movement_y /= magnitude;
        }

        state.movement = (movement_x, movement_y);

        // Action key (Space) - detect press, not hold
        let action_down = is_key_down(KeyCode::Space);
        state.action_pressed = action_down && !self.last_action_pressed;
        self.last_action_pressed = action_down;

        // Exit mech key
        state.exit_mech_pressed = is_key_pressed(KeyCode::Q);

        // Floor transition key (E for "Enter" stairway)
        state.floor_transition_pressed = is_key_pressed(KeyCode::E);

        state
    }
}

impl InputState {
    pub fn has_input(&self) -> bool {
        self.movement.0 != 0.0 || self.movement.1 != 0.0 || self.action_pressed
    }
}
