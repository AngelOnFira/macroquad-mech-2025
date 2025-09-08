use crate::network_constants::*;
use crate::{ClientMessage, TeamId, ValidationError, ValidationResult};

/// Trait for validating messages
pub trait Validate {
    fn validate(&self) -> ValidationResult<()>;
}

impl Validate for ClientMessage {
    fn validate(&self) -> ValidationResult<()> {
        match self {
            ClientMessage::JoinGame {
                player_name,
                preferred_team,
            } => {
                validate_player_name(player_name)?;
                if let Some(team) = preferred_team {
                    validate_team_id(team)?;
                }
                Ok(())
            }

            ClientMessage::PlayerInput { movement, .. } => {
                validate_movement(*movement)?;
                Ok(())
            }

            ClientMessage::StationInput { button_index } => {
                validate_button_index(*button_index)?;
                Ok(())
            }

            ClientMessage::EngineControl { movement } => {
                validate_movement(*movement)?;
                Ok(())
            }

            ClientMessage::ExitMech => Ok(()),

            ClientMessage::ExitStation => Ok(()),

            ClientMessage::FloorTransition { target_floor, .. } => {
                if *target_floor >= 3 {
                    return Err(ValidationError::InvalidFloorNumber);
                }
                Ok(())
            }

            ClientMessage::ChatMessage { message } => {
                validate_chat_message(message)?;
                Ok(())
            }
        }
    }
}

/// Validate player name
fn validate_player_name(name: &str) -> ValidationResult<()> {
    if name.is_empty() {
        return Err(ValidationError::InvalidPlayerName {
            reason: "Name cannot be empty".to_string(),
        });
    }

    if name.len() > MAX_PLAYER_NAME_LENGTH {
        return Err(ValidationError::InvalidPlayerName {
            reason: format!("Name too long (max {MAX_PLAYER_NAME_LENGTH} characters)"),
        });
    }

    // Check for valid characters (alphanumeric, spaces, underscores, hyphens)
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '_' || c == '-')
    {
        return Err(ValidationError::InvalidPlayerName {
            reason: "Name contains invalid characters".to_string(),
        });
    }

    // Check for at least one non-whitespace character
    if name.trim().is_empty() {
        return Err(ValidationError::InvalidPlayerName {
            reason: "Name must contain at least one non-whitespace character".to_string(),
        });
    }

    Ok(())
}

/// Validate team ID
fn validate_team_id(team: &TeamId) -> ValidationResult<()> {
    match team {
        TeamId::Red | TeamId::Blue => Ok(()),
    }
}

/// Validate movement vector
fn validate_movement(movement: (f32, f32)) -> ValidationResult<()> {
    let (x, y) = movement;

    // Check for NaN or infinite values
    if x.is_nan() || y.is_nan() || x.is_infinite() || y.is_infinite() {
        return Err(ValidationError::InvalidMovementNaN);
    }

    // Check reasonable bounds (movement should be normalized to -1..1 range)
    if x.abs() > MAX_MOVEMENT_MAGNITUDE || y.abs() > MAX_MOVEMENT_MAGNITUDE {
        return Err(ValidationError::InvalidMovement { x, y });
    }

    Ok(())
}

/// Validate button index for station inputs
fn validate_button_index(index: u8) -> ValidationResult<()> {
    if index >= MAX_STATION_BUTTONS {
        return Err(ValidationError::InvalidButtonIndex {
            index,
            max: MAX_STATION_BUTTONS - 1,
        });
    }
    Ok(())
}

/// Validate chat message
fn validate_chat_message(message: &str) -> ValidationResult<()> {
    if message.len() > MAX_CHAT_MESSAGE_LENGTH {
        return Err(ValidationError::MessageTooLarge {
            size: message.len(),
            max: MAX_CHAT_MESSAGE_LENGTH,
        });
    }
    Ok(())
}

/// Helper function to sanitize player names
pub fn sanitize_player_name(name: &str) -> String {
    // Trim whitespace
    let trimmed = name.trim();

    // Limit length
    let limited = if trimmed.len() > MAX_PLAYER_NAME_LENGTH {
        &trimmed[..MAX_PLAYER_NAME_LENGTH]
    } else {
        trimmed
    };

    // Remove invalid characters
    limited
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '_' || *c == '-')
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_player_name() {
        assert!(validate_player_name("ValidName").is_ok());
        assert!(validate_player_name("Valid Name 123").is_ok());
        assert!(validate_player_name("Valid_Name-123").is_ok());

        assert!(validate_player_name("").is_err());
        assert!(validate_player_name("   ").is_err());
        assert!(validate_player_name("Name@#$%").is_err());

        let long_name = "a".repeat(MAX_PLAYER_NAME_LENGTH + 1);
        assert!(validate_player_name(&long_name).is_err());
    }

    #[test]
    fn test_validate_movement() {
        assert!(validate_movement((0.0, 0.0)).is_ok());
        assert!(validate_movement((1.0, -1.0)).is_ok());
        assert!(validate_movement((0.5, 0.5)).is_ok());

        assert!(validate_movement((f32::NAN, 0.0)).is_err());
        assert!(validate_movement((0.0, f32::INFINITY)).is_err());
        assert!(validate_movement((10.0, 0.0)).is_err());
    }

    #[test]
    fn test_validate_button_index() {
        assert!(validate_button_index(0).is_ok());
        assert!(validate_button_index(MAX_STATION_BUTTONS - 1).is_ok());
        assert!(validate_button_index(MAX_STATION_BUTTONS).is_err());
    }

    #[test]
    fn test_sanitize_player_name() {
        assert_eq!(sanitize_player_name("  Valid Name  "), "Valid Name");
        assert_eq!(sanitize_player_name("Name@#$%123"), "Name123");
        assert_eq!(sanitize_player_name("   "), "");

        let long_name = "a".repeat(MAX_PLAYER_NAME_LENGTH + 10);
        assert_eq!(
            sanitize_player_name(&long_name).len(),
            MAX_PLAYER_NAME_LENGTH
        );
    }
}
