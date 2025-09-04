use super::widgets::{
    BoolWidget, EnumWidget, FloatWidget, IntWidget, KeyBindingWidget, ResolutionWidget,
    SettingValue, SettingWidget, StringWidget, WidgetConfig, KeyCode
};
use egui::Ui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type definition for a boxed setting widget
pub type BoxedSettingWidget = Box<dyn SettingWidget>;

/// Describes the metadata for a setting field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDescriptor {
    pub key: String,
    pub display_name: String,
    pub description: Option<String>,
    pub setting_type: SettingType,
    pub default_value: SettingValue,
    pub config: WidgetConfig,
    pub category: String,
    pub subcategory: Option<String>,
}

/// Enumeration of all supported setting types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettingType {
    Boolean,
    Integer,
    Float,
    Text,
    Enum { options: Vec<String> },
    KeyBinding,
    Resolution,
}

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Factory for creating setting widgets based on descriptors
pub struct WidgetFactory {
    /// Cache of created widgets by setting key
    widgets: HashMap<String, BoxedSettingWidget>,
    /// Setting descriptors by key
    descriptors: HashMap<String, SettingDescriptor>,
    /// Current values for all settings
    values: HashMap<String, SettingValue>,
    /// Whether settings have been modified
    dirty: bool,
}

impl WidgetFactory {
    /// Create a new widget factory
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
            descriptors: HashMap::new(),
            values: HashMap::new(),
            dirty: false,
        }
    }

    /// Register a setting descriptor
    pub fn register_setting(&mut self, descriptor: SettingDescriptor) {
        let key = descriptor.key.clone();
        self.values.insert(key.clone(), descriptor.default_value.clone());
        self.descriptors.insert(key, descriptor);
    }

    /// Register multiple setting descriptors at once
    pub fn register_settings(&mut self, descriptors: Vec<SettingDescriptor>) {
        for descriptor in descriptors {
            self.register_setting(descriptor);
        }
    }

    /// Create a widget for the given setting key if not already created
    pub fn get_or_create_widget(&mut self, key: &str) -> Result<&mut BoxedSettingWidget, String> {
        if !self.widgets.contains_key(key) {
            let descriptor = self.descriptors.get(key)
                .ok_or_else(|| format!("No descriptor found for setting '{}'", key))?;
            
            let widget = self.create_widget(descriptor)?;
            self.widgets.insert(key.to_string(), widget);
        }
        
        Ok(self.widgets.get_mut(key).unwrap())
    }

    /// Create a new widget based on the setting descriptor
    fn create_widget(&self, descriptor: &SettingDescriptor) -> Result<BoxedSettingWidget, String> {
        let config = descriptor.config.clone();
        
        let widget: BoxedSettingWidget = match &descriptor.setting_type {
            SettingType::Boolean => {
                match &descriptor.default_value {
                    SettingValue::Bool(default) => {
                        Box::new(BoolWidget::new(*default, config))
                    }
                    _ => return Err("Boolean setting must have boolean default value".to_string()),
                }
            }
            
            SettingType::Integer => {
                match &descriptor.default_value {
                    SettingValue::Int(default) => {
                        Box::new(IntWidget::new(*default, config))
                    }
                    _ => return Err("Integer setting must have integer default value".to_string()),
                }
            }
            
            SettingType::Float => {
                match &descriptor.default_value {
                    SettingValue::Float(default) => {
                        Box::new(FloatWidget::new(*default, config))
                    }
                    _ => return Err("Float setting must have float default value".to_string()),
                }
            }
            
            SettingType::Text => {
                match &descriptor.default_value {
                    SettingValue::String(default) => {
                        Box::new(StringWidget::new(default.clone(), config))
                    }
                    _ => return Err("Text setting must have string default value".to_string()),
                }
            }
            
            SettingType::Enum { options } => {
                match &descriptor.default_value {
                    SettingValue::Enum { selected, options: _ } => {
                        Box::new(EnumWidget::new(selected.clone(), options.clone(), config))
                    }
                    SettingValue::String(default) => {
                        // Allow string defaults for enum types
                        Box::new(EnumWidget::new(default.clone(), options.clone(), config))
                    }
                    _ => return Err("Enum setting must have enum or string default value".to_string()),
                }
            }
            
            SettingType::KeyBinding => {
                match &descriptor.default_value {
                    SettingValue::KeyBinding(default) => {
                        Box::new(KeyBindingWidget::new(default.clone(), config))
                    }
                    _ => return Err("KeyBinding setting must have KeyBinding default value".to_string()),
                }
            }
            
            SettingType::Resolution => {
                match &descriptor.default_value {
                    SettingValue::Resolution { width, height } => {
                        Box::new(ResolutionWidget::new(*width, *height, config))
                    }
                    _ => return Err("Resolution setting must have Resolution default value".to_string()),
                }
            }
        };
        
        Ok(widget)
    }

    /// Render a setting widget in the given UI
    pub fn render_setting(&mut self, ui: &mut Ui, key: &str) -> Result<bool, String> {
        let descriptor = self.descriptors.get(key)
            .ok_or_else(|| format!("No descriptor found for setting '{}'", key))?
            .clone(); // Clone to avoid borrow checker issues
        
        let widget = self.get_or_create_widget(key)?;
        
        let changed = widget.render(ui, &descriptor.display_name);
        
        // Get validation result before modifying self
        let validation = widget.validate();
        let current_value = widget.value();
        
        if changed {
            self.values.insert(key.to_string(), current_value);
            self.dirty = true;
        }
        
        // Show validation error if any
        if let Some(error_msg) = validation.error_message() {
            ui.colored_label(egui::Color32::RED, format!("⚠ {}", error_msg));
        }
        
        // Show description if available
        if let Some(ref description) = descriptor.description {
            ui.small(description);
        }
        
        Ok(changed)
    }

    /// Render all settings in a category
    pub fn render_category(&mut self, ui: &mut Ui, category: &str) -> Result<bool, String> {
        let mut any_changed = false;
        
        // Get all setting keys for this category
        let keys: Vec<String> = self.descriptors
            .values()
            .filter(|desc| desc.category == category)
            .map(|desc| desc.key.clone())
            .collect();
        
        // Group by subcategory
        let mut subcategories: HashMap<String, Vec<String>> = HashMap::new();
        
        for key in keys {
            let descriptor = self.descriptors.get(&key).unwrap();
            let subcategory = descriptor.subcategory.clone().unwrap_or_else(|| "General".to_string());
            subcategories.entry(subcategory).or_default().push(key);
        }
        
        // Render each subcategory
        for (subcategory, subcategory_keys) in subcategories {
            ui.collapsing(&subcategory, |ui| {
                for key in subcategory_keys {
                    if let Ok(changed) = self.render_setting(ui, &key) {
                        any_changed = any_changed || changed;
                    }
                    ui.separator();
                }
            });
        }
        
        Ok(any_changed)
    }

    /// Get the current value of a setting
    pub fn get_value(&self, key: &str) -> Option<&SettingValue> {
        self.values.get(key)
    }

    /// Set the value of a setting
    pub fn set_value(&mut self, key: &str, value: SettingValue) -> Result<(), String> {
        // Validate that the setting exists
        if !self.descriptors.contains_key(key) {
            return Err(format!("Unknown setting key: {}", key));
        }
        
        // If widget is already created, update it
        if let Some(widget) = self.widgets.get_mut(key) {
            widget.set_value(value.clone())?;
        }
        
        self.values.insert(key.to_string(), value);
        self.dirty = true;
        Ok(())
    }

    /// Reset a setting to its default value
    pub fn reset_setting(&mut self, key: &str) -> Result<(), String> {
        let descriptor = self.descriptors.get(key)
            .ok_or_else(|| format!("Unknown setting key: {}", key))?;
        
        let default_value = descriptor.default_value.clone();
        self.set_value(key, default_value)
    }

    /// Reset all settings in a category to their default values
    pub fn reset_category(&mut self, category: &str) -> Result<(), String> {
        let keys: Vec<String> = self.descriptors
            .values()
            .filter(|desc| desc.category == category)
            .map(|desc| desc.key.clone())
            .collect();
        
        for key in keys {
            self.reset_setting(&key)?;
        }
        
        Ok(())
    }

    /// Reset all settings to their default values
    pub fn reset_all(&mut self) -> Result<(), String> {
        let keys: Vec<String> = self.descriptors.keys().cloned().collect();
        
        for key in keys {
            self.reset_setting(&key)?;
        }
        
        Ok(())
    }

    /// Validate all current settings
    pub fn validate_all(&mut self) -> Result<Vec<ValidationError>, String> {
        let mut errors = Vec::new();
        
        let keys: Vec<String> = self.descriptors.keys().cloned().collect();
        
        for key in keys {
            let widget = self.get_or_create_widget(&key)?;
            let validation = widget.validate();
            
            if let Some(error_msg) = validation.error_message() {
                errors.push(ValidationError {
                    field: key,
                    message: error_msg.to_string(),
                });
            }
        }
        
        Ok(errors)
    }

    /// Check if any settings have been modified
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark settings as clean (e.g., after saving)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Get all current setting values as a map
    pub fn get_all_values(&self) -> HashMap<String, SettingValue> {
        self.values.clone()
    }

    /// Set multiple setting values at once (e.g., when loading from file)
    pub fn set_all_values(&mut self, values: HashMap<String, SettingValue>) -> Result<(), String> {
        for (key, value) in values {
            self.set_value(&key, value)?;
        }
        Ok(())
    }

    /// Get all available categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.descriptors
            .values()
            .map(|desc| desc.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Get all subcategories for a given category
    pub fn get_subcategories(&self, category: &str) -> Vec<String> {
        let mut subcategories: Vec<String> = self.descriptors
            .values()
            .filter(|desc| desc.category == category)
            .map(|desc| desc.subcategory.clone().unwrap_or_else(|| "General".to_string()))
            .collect();
        subcategories.sort();
        subcategories.dedup();
        subcategories
    }

    /// Get setting descriptor by key
    pub fn get_descriptor(&self, key: &str) -> Option<&SettingDescriptor> {
        self.descriptors.get(key)
    }
}

impl Default for WidgetFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create common game setting descriptors
pub fn create_default_game_settings() -> Vec<SettingDescriptor> {
    vec![
        // Graphics settings
        SettingDescriptor {
            key: "graphics.resolution".to_string(),
            display_name: "Resolution".to_string(),
            description: Some("Screen resolution for the game".to_string()),
            setting_type: SettingType::Resolution,
            default_value: SettingValue::Resolution { width: 1920, height: 1080 },
            config: WidgetConfig::default(),
            category: "Graphics".to_string(),
            subcategory: Some("Display".to_string()),
        },
        
        SettingDescriptor {
            key: "graphics.fullscreen".to_string(),
            display_name: "Fullscreen".to_string(),
            description: Some("Run the game in fullscreen mode".to_string()),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Bool(false),
            config: WidgetConfig::default(),
            category: "Graphics".to_string(),
            subcategory: Some("Display".to_string()),
        },
        
        SettingDescriptor {
            key: "graphics.vsync".to_string(),
            display_name: "V-Sync".to_string(),
            description: Some("Synchronize frame rate with display refresh rate".to_string()),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Bool(true),
            config: WidgetConfig::default(),
            category: "Graphics".to_string(),
            subcategory: Some("Display".to_string()),
        },
        
        SettingDescriptor {
            key: "graphics.render_distance".to_string(),
            display_name: "Render Distance".to_string(),
            description: Some("Maximum distance for rendering objects".to_string()),
            setting_type: SettingType::Integer,
            default_value: SettingValue::Int(100),
            config: WidgetConfig {
                min_value: Some(50.0),
                max_value: Some(500.0),
                ..Default::default()
            },
            category: "Graphics".to_string(),
            subcategory: Some("Performance".to_string()),
        },
        
        SettingDescriptor {
            key: "graphics.show_fps".to_string(),
            display_name: "Show FPS".to_string(),
            description: Some("Display frame rate counter".to_string()),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Bool(false),
            config: WidgetConfig::default(),
            category: "Graphics".to_string(),
            subcategory: Some("Performance".to_string()),
        },
        
        SettingDescriptor {
            key: "graphics.ui_scale".to_string(),
            display_name: "UI Scale".to_string(),
            description: Some("Scale factor for user interface elements".to_string()),
            setting_type: SettingType::Float,
            default_value: SettingValue::Float(1.0),
            config: WidgetConfig {
                min_value: Some(0.5),
                max_value: Some(2.0),
                step: Some(0.1),
                ..Default::default()
            },
            category: "Graphics".to_string(),
            subcategory: Some("Display".to_string()),
        },
        
        // Audio settings
        SettingDescriptor {
            key: "audio.master_volume".to_string(),
            display_name: "Master Volume".to_string(),
            description: Some("Overall volume level".to_string()),
            setting_type: SettingType::Float,
            default_value: SettingValue::Float(1.0),
            config: WidgetConfig {
                min_value: Some(0.0),
                max_value: Some(1.0),
                step: Some(0.05),
                ..Default::default()
            },
            category: "Audio".to_string(),
            subcategory: Some("Volume".to_string()),
        },
        
        SettingDescriptor {
            key: "audio.sfx_volume".to_string(),
            display_name: "Sound Effects Volume".to_string(),
            description: Some("Volume level for sound effects".to_string()),
            setting_type: SettingType::Float,
            default_value: SettingValue::Float(0.8),
            config: WidgetConfig {
                min_value: Some(0.0),
                max_value: Some(1.0),
                step: Some(0.05),
                ..Default::default()
            },
            category: "Audio".to_string(),
            subcategory: Some("Volume".to_string()),
        },
        
        SettingDescriptor {
            key: "audio.music_volume".to_string(),
            display_name: "Music Volume".to_string(),
            description: Some("Volume level for background music".to_string()),
            setting_type: SettingType::Float,
            default_value: SettingValue::Float(0.6),
            config: WidgetConfig {
                min_value: Some(0.0),
                max_value: Some(1.0),
                step: Some(0.05),
                ..Default::default()
            },
            category: "Audio".to_string(),
            subcategory: Some("Volume".to_string()),
        },
        
        SettingDescriptor {
            key: "audio.mute_when_unfocused".to_string(),
            display_name: "Mute When Unfocused".to_string(),
            description: Some("Mute audio when game window loses focus".to_string()),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Bool(true),
            config: WidgetConfig::default(),
            category: "Audio".to_string(),
            subcategory: Some("General".to_string()),
        },
        
        // Controls settings
        SettingDescriptor {
            key: "controls.mouse_sensitivity".to_string(),
            display_name: "Mouse Sensitivity".to_string(),
            description: Some("Mouse movement sensitivity".to_string()),
            setting_type: SettingType::Float,
            default_value: SettingValue::Float(1.0),
            config: WidgetConfig {
                min_value: Some(0.1),
                max_value: Some(5.0),
                step: Some(0.1),
                ..Default::default()
            },
            category: "Controls".to_string(),
            subcategory: Some("Mouse".to_string()),
        },
        
        SettingDescriptor {
            key: "controls.invert_mouse".to_string(),
            display_name: "Invert Mouse Y-Axis".to_string(),
            description: Some("Invert vertical mouse movement".to_string()),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Bool(false),
            config: WidgetConfig::default(),
            category: "Controls".to_string(),
            subcategory: Some("Mouse".to_string()),
        },
        
        SettingDescriptor {
            key: "controls.move_forward".to_string(),
            display_name: "Move Forward".to_string(),
            description: Some("Key binding for moving forward".to_string()),
            setting_type: SettingType::KeyBinding,
            default_value: SettingValue::KeyBinding(KeyCode::W),
            config: WidgetConfig::default(),
            category: "Controls".to_string(),
            subcategory: Some("Movement".to_string()),
        },
        
        SettingDescriptor {
            key: "controls.move_backward".to_string(),
            display_name: "Move Backward".to_string(),
            description: Some("Key binding for moving backward".to_string()),
            setting_type: SettingType::KeyBinding,
            default_value: SettingValue::KeyBinding(KeyCode::S),
            config: WidgetConfig::default(),
            category: "Controls".to_string(),
            subcategory: Some("Movement".to_string()),
        },
        
        SettingDescriptor {
            key: "controls.move_left".to_string(),
            display_name: "Move Left".to_string(),
            description: Some("Key binding for moving left".to_string()),
            setting_type: SettingType::KeyBinding,
            default_value: SettingValue::KeyBinding(KeyCode::A),
            config: WidgetConfig::default(),
            category: "Controls".to_string(),
            subcategory: Some("Movement".to_string()),
        },
        
        SettingDescriptor {
            key: "controls.move_right".to_string(),
            display_name: "Move Right".to_string(),
            description: Some("Key binding for moving right".to_string()),
            setting_type: SettingType::KeyBinding,
            default_value: SettingValue::KeyBinding(KeyCode::D),
            config: WidgetConfig::default(),
            category: "Controls".to_string(),
            subcategory: Some("Movement".to_string()),
        },
        
        // Network settings
        SettingDescriptor {
            key: "network.server_url".to_string(),
            display_name: "Server URL".to_string(),
            description: Some("Default server to connect to".to_string()),
            setting_type: SettingType::Text,
            default_value: SettingValue::String("ws://localhost:3000".to_string()),
            config: WidgetConfig {
                required: true,
                ..Default::default()
            },
            category: "Network".to_string(),
            subcategory: Some("Connection".to_string()),
        },
        
        SettingDescriptor {
            key: "network.auto_reconnect".to_string(),
            display_name: "Auto Reconnect".to_string(),
            description: Some("Automatically reconnect to server if disconnected".to_string()),
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Bool(true),
            config: WidgetConfig::default(),
            category: "Network".to_string(),
            subcategory: Some("Connection".to_string()),
        },
        
        SettingDescriptor {
            key: "network.connection_timeout".to_string(),
            display_name: "Connection Timeout (ms)".to_string(),
            description: Some("Maximum time to wait for server connection".to_string()),
            setting_type: SettingType::Integer,
            default_value: SettingValue::Int(5000),
            config: WidgetConfig {
                min_value: Some(1000.0),
                max_value: Some(30000.0),
                step: Some(1000.0),
                ..Default::default()
            },
            category: "Network".to_string(),
            subcategory: Some("Connection".to_string()),
        },
        
        SettingDescriptor {
            key: "network.preferred_name".to_string(),
            display_name: "Player Name".to_string(),
            description: Some("Default player name for multiplayer games".to_string()),
            setting_type: SettingType::Text,
            default_value: SettingValue::String("Player".to_string()),
            config: WidgetConfig {
                required: true,
                ..Default::default()
            },
            category: "Network".to_string(),
            subcategory: Some("Player".to_string()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_factory_creation() {
        let factory = WidgetFactory::new();
        assert!(!factory.is_dirty());
        assert!(factory.get_categories().is_empty());
    }

    #[test]
    fn test_register_and_create_widget() {
        let mut factory = WidgetFactory::new();
        
        let descriptor = SettingDescriptor {
            key: "test.bool".to_string(),
            display_name: "Test Boolean".to_string(),
            description: None,
            setting_type: SettingType::Boolean,
            default_value: SettingValue::Bool(true),
            config: WidgetConfig::default(),
            category: "Test".to_string(),
            subcategory: None,
        };
        
        factory.register_setting(descriptor);
        assert_eq!(factory.get_categories(), vec!["Test"]);
        
        // Widget should be created on first access
        let widget_result = factory.get_or_create_widget("test.bool");
        assert!(widget_result.is_ok());
        
        assert_eq!(factory.get_value("test.bool"), Some(&SettingValue::Bool(true)));
    }

    #[test]
    fn test_set_and_get_value() {
        let mut factory = WidgetFactory::new();
        
        let descriptor = SettingDescriptor {
            key: "test.int".to_string(),
            display_name: "Test Integer".to_string(),
            description: None,
            setting_type: SettingType::Integer,
            default_value: SettingValue::Int(42),
            config: WidgetConfig::default(),
            category: "Test".to_string(),
            subcategory: None,
        };
        
        factory.register_setting(descriptor);
        
        assert_eq!(factory.get_value("test.int"), Some(&SettingValue::Int(42)));
        
        factory.set_value("test.int", SettingValue::Int(100)).unwrap();
        assert_eq!(factory.get_value("test.int"), Some(&SettingValue::Int(100)));
        assert!(factory.is_dirty());
    }

    #[test]
    fn test_reset_to_default() {
        let mut factory = WidgetFactory::new();
        
        let descriptor = SettingDescriptor {
            key: "test.string".to_string(),
            display_name: "Test String".to_string(),
            description: None,
            setting_type: SettingType::Text,
            default_value: SettingValue::String("default".to_string()),
            config: WidgetConfig::default(),
            category: "Test".to_string(),
            subcategory: None,
        };
        
        factory.register_setting(descriptor);
        
        factory.set_value("test.string", SettingValue::String("modified".to_string())).unwrap();
        assert_eq!(factory.get_value("test.string"), Some(&SettingValue::String("modified".to_string())));
        
        factory.reset_setting("test.string").unwrap();
        assert_eq!(factory.get_value("test.string"), Some(&SettingValue::String("default".to_string())));
    }

    #[test]
    fn test_validation() {
        let mut factory = WidgetFactory::new();
        
        let descriptor = SettingDescriptor {
            key: "test.required_string".to_string(),
            display_name: "Required String".to_string(),
            description: None,
            setting_type: SettingType::Text,
            default_value: SettingValue::String("valid".to_string()),
            config: WidgetConfig {
                required: true,
                ..Default::default()
            },
            category: "Test".to_string(),
            subcategory: None,
        };
        
        factory.register_setting(descriptor);
        
        // Valid value
        let errors = factory.validate_all().unwrap();
        assert!(errors.is_empty());
        
        // Invalid value (empty string)
        factory.set_value("test.required_string", SettingValue::String("".to_string())).unwrap();
        let errors = factory.validate_all().unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "test.required_string");
    }

    #[test]
    fn test_default_game_settings() {
        let settings = create_default_game_settings();
        assert!(!settings.is_empty());
        
        // Verify we have settings in all major categories
        let categories: Vec<&str> = settings.iter().map(|s| s.category.as_str()).collect();
        assert!(categories.contains(&"Graphics"));
        assert!(categories.contains(&"Audio"));
        assert!(categories.contains(&"Controls"));
        assert!(categories.contains(&"Network"));
    }

    #[test]
    fn test_category_operations() {
        let mut factory = WidgetFactory::new();
        factory.register_settings(create_default_game_settings());
        
        let categories = factory.get_categories();
        assert!(categories.contains(&"Graphics".to_string()));
        assert!(categories.contains(&"Audio".to_string()));
        assert!(categories.contains(&"Controls".to_string()));
        assert!(categories.contains(&"Network".to_string()));
        
        let graphics_subcategories = factory.get_subcategories("Graphics");
        assert!(graphics_subcategories.contains(&"Display".to_string()));
        assert!(graphics_subcategories.contains(&"Performance".to_string()));
    }
}