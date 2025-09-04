//! Settings system for the Mech Battle Arena client
//!
//! This module provides a comprehensive settings management system with:
//! - Widget-based UI components for different setting types
//! - Factory pattern for dynamic widget creation  
//! - Validation and error handling
//! - Category-based organization
//! - Persistence integration (to be implemented)
//!
//! # Architecture
//!
//! The settings system is built around these core components:
//!
//! ## Widgets (`widgets.rs`)
//! - `SettingWidget` trait: Base interface for all setting controls
//! - Specialized widget implementations for different data types:
//!   - `BoolWidget`: Checkboxes for boolean values
//!   - `IntWidget`/`FloatWidget`: Sliders and drag values for numbers
//!   - `StringWidget`: Text input fields
//!   - `EnumWidget`: Dropdown/combo boxes for selections
//!   - `KeyBindingWidget`: Click-to-capture key binding controls
//!   - `ResolutionWidget`: Custom resolution picker with presets
//!
//! ## Factory (`widget_factory.rs`)
//! - `WidgetFactory`: Creates and manages widget instances
//! - `SettingDescriptor`: Metadata for configuring widgets
//! - Dynamic widget creation based on setting type
//! - Category/subcategory organization
//! - Validation and error handling
//!
//! ## Integration
//! This module is designed to integrate with:
//! - Settings window UI (handled by Stream A)
//! - Settings data model and persistence (Task 4)
//! - Main menu system (Task 5)
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use crate::settings::widget_factory::{WidgetFactory, create_default_game_settings};
//!
//! let mut factory = WidgetFactory::new();
//! factory.register_settings(create_default_game_settings());
//!
//! // In UI rendering code:
//! // factory.render_category(ui, "Graphics");
//! ```

pub mod widgets;
pub mod widget_factory;

// Re-export commonly used types
pub use widgets::{
    SettingValue, SettingWidget, ValidationResult, WidgetConfig, KeyCode
};
pub use widget_factory::{
    WidgetFactory, SettingDescriptor, SettingType, ValidationError, 
    BoxedSettingWidget, create_default_game_settings
};