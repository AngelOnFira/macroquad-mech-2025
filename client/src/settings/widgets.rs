use egui::{ComboBox, DragValue, Slider, TextEdit, Ui};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Represents different types of setting values that can be configured
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Enum { selected: String, options: Vec<String> },
    KeyBinding(KeyCode),
    Resolution { width: u32, height: u32 },
}

/// Key code enumeration for key binding settings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Special keys
    Space, Enter, Escape, Tab, Backspace, Delete, Insert, Home, End, PageUp, PageDown,
    Left, Right, Up, Down,
    LeftShift, RightShift, LeftCtrl, RightCtrl, LeftAlt, RightAlt,
    // Mouse buttons
    LeftMouse, RightMouse, MiddleMouse,
}

impl Display for KeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            KeyCode::A => "A", KeyCode::B => "B", KeyCode::C => "C", KeyCode::D => "D",
            KeyCode::E => "E", KeyCode::F => "F", KeyCode::G => "G", KeyCode::H => "H",
            KeyCode::I => "I", KeyCode::J => "J", KeyCode::K => "K", KeyCode::L => "L",
            KeyCode::M => "M", KeyCode::N => "N", KeyCode::O => "O", KeyCode::P => "P",
            KeyCode::Q => "Q", KeyCode::R => "R", KeyCode::S => "S", KeyCode::T => "T",
            KeyCode::U => "U", KeyCode::V => "V", KeyCode::W => "W", KeyCode::X => "X",
            KeyCode::Y => "Y", KeyCode::Z => "Z",
            
            KeyCode::Key0 => "0", KeyCode::Key1 => "1", KeyCode::Key2 => "2", KeyCode::Key3 => "3",
            KeyCode::Key4 => "4", KeyCode::Key5 => "5", KeyCode::Key6 => "6", KeyCode::Key7 => "7",
            KeyCode::Key8 => "8", KeyCode::Key9 => "9",
            
            KeyCode::F1 => "F1", KeyCode::F2 => "F2", KeyCode::F3 => "F3", KeyCode::F4 => "F4",
            KeyCode::F5 => "F5", KeyCode::F6 => "F6", KeyCode::F7 => "F7", KeyCode::F8 => "F8",
            KeyCode::F9 => "F9", KeyCode::F10 => "F10", KeyCode::F11 => "F11", KeyCode::F12 => "F12",
            
            KeyCode::Space => "Space", KeyCode::Enter => "Enter", KeyCode::Escape => "Escape",
            KeyCode::Tab => "Tab", KeyCode::Backspace => "Backspace", KeyCode::Delete => "Delete",
            KeyCode::Insert => "Insert", KeyCode::Home => "Home", KeyCode::End => "End",
            KeyCode::PageUp => "Page Up", KeyCode::PageDown => "Page Down",
            KeyCode::Left => "←", KeyCode::Right => "→", KeyCode::Up => "↑", KeyCode::Down => "↓",
            KeyCode::LeftShift => "Left Shift", KeyCode::RightShift => "Right Shift",
            KeyCode::LeftCtrl => "Left Ctrl", KeyCode::RightCtrl => "Right Ctrl",
            KeyCode::LeftAlt => "Left Alt", KeyCode::RightAlt => "Right Alt",
            KeyCode::LeftMouse => "Left Click", KeyCode::RightMouse => "Right Click",
            KeyCode::MiddleMouse => "Middle Click",
        };
        write!(f, "{}", text)
    }
}

/// Validation result for setting values
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            ValidationResult::Valid => None,
            ValidationResult::Invalid(msg) => Some(msg),
        }
    }
}

/// Configuration for widget constraints and validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    /// Minimum value for numeric widgets
    pub min_value: Option<f64>,
    /// Maximum value for numeric widgets
    pub max_value: Option<f64>,
    /// Step size for numeric widgets
    pub step: Option<f64>,
    /// Whether the value is required (cannot be empty for strings)
    pub required: bool,
    /// Custom validation function name for complex validation
    pub validator: Option<String>,
}

impl Default for WidgetConfig {
    fn default() -> Self {
        Self {
            min_value: None,
            max_value: None,
            step: None,
            required: false,
            validator: None,
        }
    }
}

/// Base trait for all setting widgets
pub trait SettingWidget {
    /// Render the widget in the given UI context and return whether the value changed
    fn render(&mut self, ui: &mut Ui, label: &str) -> bool;
    
    /// Get the current value of the widget
    fn value(&self) -> SettingValue;
    
    /// Set the value of the widget (for initialization and reset)
    fn set_value(&mut self, value: SettingValue) -> Result<(), String>;
    
    /// Validate the current value
    fn validate(&self) -> ValidationResult;
    
    /// Reset to default value
    fn reset_to_default(&mut self);
    
    /// Check if the current value is the default value
    fn is_default(&self) -> bool;
}

/// Boolean widget implementation (checkbox)
pub struct BoolWidget {
    value: bool,
    default_value: bool,
    config: WidgetConfig,
}

impl BoolWidget {
    pub fn new(default_value: bool, config: WidgetConfig) -> Self {
        Self {
            value: default_value,
            default_value,
            config,
        }
    }
}

impl SettingWidget for BoolWidget {
    fn render(&mut self, ui: &mut Ui, label: &str) -> bool {
        let response = ui.checkbox(&mut self.value, label);
        response.changed()
    }
    
    fn value(&self) -> SettingValue {
        SettingValue::Bool(self.value)
    }
    
    fn set_value(&mut self, value: SettingValue) -> Result<(), String> {
        match value {
            SettingValue::Bool(v) => {
                self.value = v;
                Ok(())
            }
            _ => Err("Invalid value type for boolean widget".to_string()),
        }
    }
    
    fn validate(&self) -> ValidationResult {
        ValidationResult::Valid // Booleans are always valid
    }
    
    fn reset_to_default(&mut self) {
        self.value = self.default_value;
    }
    
    fn is_default(&self) -> bool {
        self.value == self.default_value
    }
}

/// Integer widget implementation (slider or text input)
pub struct IntWidget {
    value: i32,
    default_value: i32,
    config: WidgetConfig,
}

impl IntWidget {
    pub fn new(default_value: i32, config: WidgetConfig) -> Self {
        Self {
            value: default_value,
            default_value,
            config,
        }
    }
}

impl SettingWidget for IntWidget {
    fn render(&mut self, ui: &mut Ui, label: &str) -> bool {
        ui.horizontal(|ui| {
            ui.label(label);
            
            let min = self.config.min_value.unwrap_or(i32::MIN as f64) as i32;
            let max = self.config.max_value.unwrap_or(i32::MAX as f64) as i32;
            
            // Use slider for bounded values, drag value for unbounded
            if self.config.min_value.is_some() && self.config.max_value.is_some() {
                ui.add(Slider::new(&mut self.value, min..=max))
            } else {
                ui.add(DragValue::new(&mut self.value).range(min..=max))
            }
        }).inner.changed()
    }
    
    fn value(&self) -> SettingValue {
        SettingValue::Int(self.value)
    }
    
    fn set_value(&mut self, value: SettingValue) -> Result<(), String> {
        match value {
            SettingValue::Int(v) => {
                self.value = v;
                Ok(())
            }
            _ => Err("Invalid value type for integer widget".to_string()),
        }
    }
    
    fn validate(&self) -> ValidationResult {
        if let Some(min) = self.config.min_value {
            if (self.value as f64) < min {
                return ValidationResult::Invalid(format!("Value must be at least {}", min));
            }
        }
        
        if let Some(max) = self.config.max_value {
            if (self.value as f64) > max {
                return ValidationResult::Invalid(format!("Value must be at most {}", max));
            }
        }
        
        ValidationResult::Valid
    }
    
    fn reset_to_default(&mut self) {
        self.value = self.default_value;
    }
    
    fn is_default(&self) -> bool {
        self.value == self.default_value
    }
}

/// Float widget implementation (slider or text input)
pub struct FloatWidget {
    value: f32,
    default_value: f32,
    config: WidgetConfig,
}

impl FloatWidget {
    pub fn new(default_value: f32, config: WidgetConfig) -> Self {
        Self {
            value: default_value,
            default_value,
            config,
        }
    }
}

impl SettingWidget for FloatWidget {
    fn render(&mut self, ui: &mut Ui, label: &str) -> bool {
        ui.horizontal(|ui| {
            ui.label(label);
            
            let min = self.config.min_value.unwrap_or(f32::NEG_INFINITY as f64) as f32;
            let max = self.config.max_value.unwrap_or(f32::INFINITY as f64) as f32;
            
            // Use slider for bounded values, drag value for unbounded
            if self.config.min_value.is_some() && self.config.max_value.is_some() {
                ui.add(Slider::new(&mut self.value, min..=max))
            } else {
                ui.add(DragValue::new(&mut self.value).range(min..=max).speed(0.1))
            }
        }).inner.changed()
    }
    
    fn value(&self) -> SettingValue {
        SettingValue::Float(self.value)
    }
    
    fn set_value(&mut self, value: SettingValue) -> Result<(), String> {
        match value {
            SettingValue::Float(v) => {
                self.value = v;
                Ok(())
            }
            _ => Err("Invalid value type for float widget".to_string()),
        }
    }
    
    fn validate(&self) -> ValidationResult {
        if self.value.is_nan() || self.value.is_infinite() {
            return ValidationResult::Invalid("Value must be a valid number".to_string());
        }
        
        if let Some(min) = self.config.min_value {
            if (self.value as f64) < min {
                return ValidationResult::Invalid(format!("Value must be at least {}", min));
            }
        }
        
        if let Some(max) = self.config.max_value {
            if (self.value as f64) > max {
                return ValidationResult::Invalid(format!("Value must be at most {}", max));
            }
        }
        
        ValidationResult::Valid
    }
    
    fn reset_to_default(&mut self) {
        self.value = self.default_value;
    }
    
    fn is_default(&self) -> bool {
        (self.value - self.default_value).abs() < f32::EPSILON
    }
}

/// String widget implementation (text input)
pub struct StringWidget {
    value: String,
    default_value: String,
    config: WidgetConfig,
}

impl StringWidget {
    pub fn new(default_value: String, config: WidgetConfig) -> Self {
        Self {
            value: default_value.clone(),
            default_value,
            config,
        }
    }
}

impl SettingWidget for StringWidget {
    fn render(&mut self, ui: &mut Ui, label: &str) -> bool {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add(TextEdit::singleline(&mut self.value))
        }).inner.changed()
    }
    
    fn value(&self) -> SettingValue {
        SettingValue::String(self.value.clone())
    }
    
    fn set_value(&mut self, value: SettingValue) -> Result<(), String> {
        match value {
            SettingValue::String(v) => {
                self.value = v;
                Ok(())
            }
            _ => Err("Invalid value type for string widget".to_string()),
        }
    }
    
    fn validate(&self) -> ValidationResult {
        if self.config.required && self.value.trim().is_empty() {
            return ValidationResult::Invalid("This field is required".to_string());
        }
        
        ValidationResult::Valid
    }
    
    fn reset_to_default(&mut self) {
        self.value = self.default_value.clone();
    }
    
    fn is_default(&self) -> bool {
        self.value == self.default_value
    }
}

/// Enum widget implementation (combo box)
pub struct EnumWidget {
    selected: String,
    options: Vec<String>,
    default_value: String,
    config: WidgetConfig,
}

impl EnumWidget {
    pub fn new(default_value: String, options: Vec<String>, config: WidgetConfig) -> Self {
        Self {
            selected: default_value.clone(),
            options,
            default_value,
            config,
        }
    }
}

impl SettingWidget for EnumWidget {
    fn render(&mut self, ui: &mut Ui, label: &str) -> bool {
        ui.horizontal(|ui| {
            ui.label(label);
            ComboBox::from_id_salt(label)
                .selected_text(&self.selected)
                .show_ui(ui, |ui| {
                    let mut changed = false;
                    for option in &self.options {
                        if ui.selectable_value(&mut self.selected, option.clone(), option).changed() {
                            changed = true;
                        }
                    }
                    changed
                })
        }).inner.inner.unwrap_or(false)
    }
    
    fn value(&self) -> SettingValue {
        SettingValue::Enum {
            selected: self.selected.clone(),
            options: self.options.clone(),
        }
    }
    
    fn set_value(&mut self, value: SettingValue) -> Result<(), String> {
        match value {
            SettingValue::Enum { selected, options } => {
                if !options.contains(&selected) {
                    return Err("Selected value not in options".to_string());
                }
                self.selected = selected;
                self.options = options;
                Ok(())
            }
            _ => Err("Invalid value type for enum widget".to_string()),
        }
    }
    
    fn validate(&self) -> ValidationResult {
        if !self.options.contains(&self.selected) {
            return ValidationResult::Invalid("Selected value not in available options".to_string());
        }
        
        ValidationResult::Valid
    }
    
    fn reset_to_default(&mut self) {
        self.selected = self.default_value.clone();
    }
    
    fn is_default(&self) -> bool {
        self.selected == self.default_value
    }
}

/// Key binding widget implementation (click to capture)
pub struct KeyBindingWidget {
    value: KeyCode,
    default_value: KeyCode,
    capturing: bool,
    config: WidgetConfig,
}

impl KeyBindingWidget {
    pub fn new(default_value: KeyCode, config: WidgetConfig) -> Self {
        Self {
            value: default_value.clone(),
            default_value,
            capturing: false,
            config,
        }
    }
}

impl SettingWidget for KeyBindingWidget {
    fn render(&mut self, ui: &mut Ui, label: &str) -> bool {
        ui.horizontal(|ui| {
            ui.label(label);
            
            let button_text = if self.capturing {
                "Press any key...".to_string()
            } else {
                format!("{}", self.value)
            };
            
            if ui.button(button_text).clicked() {
                self.capturing = !self.capturing;
            }
            
            // TODO: In a real implementation, this would need to hook into
            // the input system to capture key presses when capturing is true
            false // For now, always return false as we don't have key capture
        }).inner
    }
    
    fn value(&self) -> SettingValue {
        SettingValue::KeyBinding(self.value.clone())
    }
    
    fn set_value(&mut self, value: SettingValue) -> Result<(), String> {
        match value {
            SettingValue::KeyBinding(v) => {
                self.value = v;
                Ok(())
            }
            _ => Err("Invalid value type for key binding widget".to_string()),
        }
    }
    
    fn validate(&self) -> ValidationResult {
        ValidationResult::Valid // All key codes are valid
    }
    
    fn reset_to_default(&mut self) {
        self.value = self.default_value.clone();
    }
    
    fn is_default(&self) -> bool {
        self.value == self.default_value
    }
}

/// Resolution widget implementation (custom width x height picker)
pub struct ResolutionWidget {
    width: u32,
    height: u32,
    default_width: u32,
    default_height: u32,
    predefined_resolutions: Vec<(u32, u32, &'static str)>,
    config: WidgetConfig,
}

impl ResolutionWidget {
    pub fn new(default_width: u32, default_height: u32, config: WidgetConfig) -> Self {
        Self {
            width: default_width,
            height: default_height,
            default_width,
            default_height,
            predefined_resolutions: vec![
                (1920, 1080, "1080p"),
                (2560, 1440, "1440p"),
                (3840, 2160, "4K"),
                (1366, 768, "768p"),
                (1600, 900, "900p"),
                (1280, 720, "720p"),
            ],
            config,
        }
    }
}

impl SettingWidget for ResolutionWidget {
    fn render(&mut self, ui: &mut Ui, label: &str) -> bool {
        ui.vertical(|ui| {
            ui.label(label);
            
            let mut changed = false;
            
            // Predefined resolutions
            ui.horizontal(|ui| {
                ui.label("Preset:");
                for (w, h, name) in &self.predefined_resolutions {
                    if ui.small_button(*name).clicked() {
                        self.width = *w;
                        self.height = *h;
                        changed = true;
                    }
                }
            });
            
            // Custom resolution
            ui.horizontal(|ui| {
                ui.label("Custom:");
                let mut width_i32 = self.width as i32;
                let mut height_i32 = self.height as i32;
                
                if ui.add(DragValue::new(&mut width_i32).range(640..=7680)).changed() {
                    self.width = width_i32 as u32;
                    changed = true;
                }
                
                ui.label("×");
                
                if ui.add(DragValue::new(&mut height_i32).range(480..=4320)).changed() {
                    self.height = height_i32 as u32;
                    changed = true;
                }
            });
            
            changed
        }).inner
    }
    
    fn value(&self) -> SettingValue {
        SettingValue::Resolution {
            width: self.width,
            height: self.height,
        }
    }
    
    fn set_value(&mut self, value: SettingValue) -> Result<(), String> {
        match value {
            SettingValue::Resolution { width, height } => {
                self.width = width;
                self.height = height;
                Ok(())
            }
            _ => Err("Invalid value type for resolution widget".to_string()),
        }
    }
    
    fn validate(&self) -> ValidationResult {
        if self.width < 640 || self.width > 7680 {
            return ValidationResult::Invalid("Width must be between 640 and 7680 pixels".to_string());
        }
        
        if self.height < 480 || self.height > 4320 {
            return ValidationResult::Invalid("Height must be between 480 and 4320 pixels".to_string());
        }
        
        ValidationResult::Valid
    }
    
    fn reset_to_default(&mut self) {
        self.width = self.default_width;
        self.height = self.default_height;
    }
    
    fn is_default(&self) -> bool {
        self.width == self.default_width && self.height == self.default_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_widget() {
        let mut widget = BoolWidget::new(false, WidgetConfig::default());
        
        assert_eq!(widget.value(), SettingValue::Bool(false));
        assert!(widget.is_default());
        assert!(widget.validate().is_valid());
        
        widget.set_value(SettingValue::Bool(true)).unwrap();
        assert_eq!(widget.value(), SettingValue::Bool(true));
        assert!(!widget.is_default());
        
        widget.reset_to_default();
        assert!(widget.is_default());
    }

    #[test]
    fn test_int_widget_validation() {
        let config = WidgetConfig {
            min_value: Some(0.0),
            max_value: Some(100.0),
            ..Default::default()
        };
        let mut widget = IntWidget::new(50, config);
        
        assert!(widget.validate().is_valid());
        
        widget.value = -10;
        assert!(!widget.validate().is_valid());
        
        widget.value = 150;
        assert!(!widget.validate().is_valid());
        
        widget.value = 75;
        assert!(widget.validate().is_valid());
    }

    #[test]
    fn test_float_widget_nan_handling() {
        let mut widget = FloatWidget::new(1.0, WidgetConfig::default());
        
        widget.value = f32::NAN;
        assert!(!widget.validate().is_valid());
        
        widget.value = f32::INFINITY;
        assert!(!widget.validate().is_valid());
        
        widget.value = 42.5;
        assert!(widget.validate().is_valid());
    }

    #[test]
    fn test_string_widget_required() {
        let config = WidgetConfig {
            required: true,
            ..Default::default()
        };
        let mut widget = StringWidget::new("default".to_string(), config);
        
        assert!(widget.validate().is_valid());
        
        widget.value = "".to_string();
        assert!(!widget.validate().is_valid());
        
        widget.value = "   ".to_string(); // Only whitespace
        assert!(!widget.validate().is_valid());
        
        widget.value = "valid".to_string();
        assert!(widget.validate().is_valid());
    }

    #[test]
    fn test_enum_widget() {
        let options = vec!["Low".to_string(), "Medium".to_string(), "High".to_string()];
        let mut widget = EnumWidget::new("Medium".to_string(), options.clone(), WidgetConfig::default());
        
        assert!(widget.validate().is_valid());
        
        // Test invalid selection
        widget.selected = "Invalid".to_string();
        assert!(!widget.validate().is_valid());
        
        // Test valid selection
        widget.selected = "High".to_string();
        assert!(widget.validate().is_valid());
    }

    #[test]
    fn test_resolution_widget_validation() {
        let mut widget = ResolutionWidget::new(1920, 1080, WidgetConfig::default());
        
        assert!(widget.validate().is_valid());
        
        widget.width = 500; // Too small
        assert!(!widget.validate().is_valid());
        
        widget.width = 1920;
        widget.height = 400; // Too small
        assert!(!widget.validate().is_valid());
        
        widget.height = 1080;
        assert!(widget.validate().is_valid());
    }

    #[test]
    fn test_setting_value_types() {
        let bool_val = SettingValue::Bool(true);
        let int_val = SettingValue::Int(42);
        let float_val = SettingValue::Float(3.14);
        let string_val = SettingValue::String("test".to_string());
        let resolution_val = SettingValue::Resolution { width: 1920, height: 1080 };
        
        // Test serialization/deserialization
        assert_eq!(serde_json::from_str::<SettingValue>(&serde_json::to_string(&bool_val).unwrap()).unwrap(), bool_val);
        assert_eq!(serde_json::from_str::<SettingValue>(&serde_json::to_string(&int_val).unwrap()).unwrap(), int_val);
        assert_eq!(serde_json::from_str::<SettingValue>(&serde_json::to_string(&float_val).unwrap()).unwrap(), float_val);
        assert_eq!(serde_json::from_str::<SettingValue>(&serde_json::to_string(&string_val).unwrap()).unwrap(), string_val);
        assert_eq!(serde_json::from_str::<SettingValue>(&serde_json::to_string(&resolution_val).unwrap()).unwrap(), resolution_val);
    }

    #[test]
    fn test_keycode_display() {
        assert_eq!(format!("{}", KeyCode::A), "A");
        assert_eq!(format!("{}", KeyCode::Space), "Space");
        assert_eq!(format!("{}", KeyCode::F1), "F1");
        assert_eq!(format!("{}", KeyCode::LeftMouse), "Left Click");
        assert_eq!(format!("{}", KeyCode::Up), "↑");
    }
}