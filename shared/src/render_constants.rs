// Rendering and UI constants

// ===== Visual Effects =====
pub const ARENA_BOUNDARY_WIDTH: f32 = 3.0;
pub const MECH_OUTLINE_WIDTH: f32 = 2.0;
pub const LASER_BEAM_WIDTH: f32 = 3.0;
pub const OXYGEN_TETHER_WIDTH: f32 = 2.0;
pub const OXYGEN_TETHER_OPACITY: f32 = 0.6;
pub const OXYGEN_DANGER_DISTANCE: f32 = 10.0; // tiles
pub const WEAPON_EFFECT_DURATION: f32 = 1.0; // seconds
pub const GRASS_VARIATION: f32 = 0.02;

// ===== Colors (RGB values) =====
pub const OXYGEN_DANGER_COLOR: (f32, f32, f32) = (0.8, 0.2, 0.2);
pub const TEAM_RED_COLOR: (f32, f32, f32, f32) = (0.8, 0.2, 0.2, 1.0);
pub const TEAM_BLUE_COLOR: (f32, f32, f32, f32) = (0.2, 0.2, 0.8, 1.0);
pub const TEAM_RED_PLAYER_COLOR: (f32, f32, f32, f32) = (1.0, 0.5, 0.5, 1.0);
pub const TEAM_BLUE_PLAYER_COLOR: (f32, f32, f32, f32) = (0.5, 0.5, 1.0, 1.0);
pub const DOOR_INTERIOR_COLOR: (f32, f32, f32, f32) = (0.3, 0.3, 0.3, 1.0);
pub const LASER_BEAM_COLOR: (f32, f32, f32, f32) = (1.0, 0.0, 0.0, 1.0);
pub const GRASS_BASE_COLOR: (f32, f32, f32, f32) = (0.2, 0.4, 0.2, 1.0);
pub const GRASS_GRID_COLOR: (f32, f32, f32, f32) = (0.15, 0.3, 0.15, 1.0);
pub const STATION_OVERLAY_COLOR: (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 0.8);

// Radar colors
pub const RADAR_BG_COLOR: (f32, f32, f32, f32) = (0.05, 0.1, 0.05, 1.0);
pub const RADAR_GRID_COLOR: (f32, f32, f32, f32) = (0.0, 0.3, 0.0, 0.3);
pub const RADAR_SCAN_COLOR: (f32, f32, f32, f32) = (0.0, 1.0, 0.0, 0.2);
pub const RADAR_PLAYER_MECH_COLOR: (f32, f32, f32, f32) = (0.0, 1.0, 0.0, 1.0);
pub const RADAR_ENEMY_MECH_COLOR: (f32, f32, f32, f32) = (1.0, 0.0, 0.0, 1.0);

// ===== Mech Door Visuals =====
pub const DOOR_WIDTH_MULTIPLIER: f32 = 1.5;
pub const DOOR_HEIGHT_MULTIPLIER: f32 = 2.0;
pub const DOOR_OVERLAP_RATIO: f32 = 0.2;
pub const DOOR_INTERIOR_PADDING: f32 = 4.0;
pub const TEAM_COLOR_STRIP_HEIGHT: f32 = 8.0;
pub const DOOR_THIRD_DIVISOR: f32 = 3.0;

// ===== Entity Rendering =====
pub const RESOURCE_CIRCLE_RADIUS_DIVISOR: f32 = 3.0; // TILE_SIZE / 3.0
pub const PROJECTILE_RADIUS: f32 = 5.0;
pub const LADDER_CIRCLE_RADIUS_DIVISOR: f32 = 4.0; // TILE_SIZE / 4.0
pub const PLAYER_CIRCLE_RADIUS_DIVISOR: f32 = 5.0; // TILE_SIZE / 5.0

// ===== Font Sizes =====
pub const DOOR_LABEL_FONT_SIZE: f32 = 12.0;
pub const LADDER_TEXT_SIZE: f32 = 12.0;
pub const HUD_FONT_SIZE: f32 = 20.0;
pub const STATION_TITLE_FONT_SIZE: f32 = 30.0;
pub const STATION_BUTTON_FONT_SIZE: f32 = 20.0;
pub const CONNECTION_STATUS_FONT_SIZE: f32 = 30.0;
pub const PLAYER_NAME_FONT_SIZE: f32 = 12.0;
pub const STATION_LABEL_FONT_SIZE: f32 = 10.0;
pub const MECH_STATUS_FONT_SIZE: f32 = 14.0;
pub const UI_TEXT_FONT_SIZE: f32 = 18.0;
pub const SMALL_TEXT_FONT_SIZE: f32 = 16.0;
pub const COMPASS_FONT_SIZE: f32 = 14.0;

// ===== HUD Positioning =====
pub const HUD_BASE_X: f32 = 10.0;
pub const HUD_BASE_Y: f32 = 50.0;
pub const HUD_LINE_SPACING: f32 = 25.0;
pub const HUD_TEXT_OFFSET_X: f32 = 20.0;
pub const HUD_TEXT_OFFSET_Y: f32 = 20.0;
pub const HUD_STATUS_SPACING: f32 = 30.0;

// ===== Station Interface =====
pub const STATION_PANEL_WIDTH_RATIO: f32 = 0.3;
pub const STATION_PANEL_MIN_WIDTH: f32 = 300.0;
pub const STATION_PANEL_HEIGHT: f32 = 400.0;
pub const STATION_PANEL_PADDING: f32 = 20.0;
pub const STATION_BUTTON_HEIGHT: f32 = 60.0;
pub const STATION_BAR_WIDTH: f32 = 100.0;
pub const STATION_BUTTON_WIDTH: f32 = 200.0;
pub const STATION_BUTTON_OFFSET_X: f32 = 10.0;
pub const STATION_BUTTON_SPACING: f32 = 70.0;
pub const STATION_EXIT_TEXT_OFFSET: f32 = 30.0;

// Station Y offsets
pub const STATION_TITLE_Y_OFFSET: f32 = 30.0;
pub const STATION_INFO_Y_OFFSET: f32 = 70.0;
pub const STATION_BUTTONS_Y_OFFSET: f32 = 150.0;
pub const ENGINE_RADAR_Y_OFFSET: f32 = 180.0;
pub const SHIELD_STATUS_Y_OFFSET: f32 = 100.0;
pub const SHIELD_BAR_Y_OFFSET: f32 = 130.0;

// ===== Resource Panel =====
pub const RESOURCE_PANEL_WIDTH: f32 = 200.0;
pub const RESOURCE_PANEL_HEIGHT: f32 = 120.0;
pub const RESOURCE_PANEL_OFFSET_X: f32 = 10.0;
pub const RESOURCE_PANEL_OFFSET_Y: f32 = 100.0;
pub const RESOURCE_TEXT_SPACING: f32 = 20.0;

// ===== Radar Display =====
pub const RADAR_SIZE: f32 = 150.0;
pub const RADAR_OFFSET_X: f32 = 50.0;
pub const RADAR_OFFSET_Y: f32 = 30.0;
pub const RADAR_GRID_SIZE: i32 = 5;
pub const RADAR_RANGE_TILES: f32 = 50.0;
pub const RADAR_SWEEP_SPEED: f32 = 2.0;

// ===== UI Positioning =====
pub const CONNECTION_MESSAGE_X: f32 = 10.0;
pub const CONNECTION_MESSAGE_Y: f32 = 30.0;
pub const SCREEN_WIDTH_DIVISOR: f32 = 3.0;
pub const WINDOW_CENTER_DIVISOR: f32 = 2.0;

// ===== Health/Shield Bars =====
pub const HEALTH_BAR_WIDTH: f32 = 80.0;
pub const HEALTH_BAR_HEIGHT: f32 = 10.0;
pub const BAR_OFFSET_Y: f32 = 15.0;

// ===== Tether Distance Display =====
pub const TETHER_DISTANCE_OFFSET_Y: f32 = 30.0;
pub const OXYGEN_LOW_THRESHOLD: f32 = 5.0;

// ===== Additional UI Constants =====
pub const GRID_LINE_WIDTH: f32 = 1.0;
pub const RADAR_COMPASS_COLOR: (f32, f32, f32, f32) = (0.0, 0.8, 0.0, 0.8);
pub const RADAR_COMPASS_EDGE_OFFSET: f32 = 15.0;
pub const RADAR_COMPASS_Y_OFFSET: f32 = 5.0;
pub const RADAR_CENTER_DIVISOR: f32 = 2.0;
pub const REPAIR_STATION_Y_OFFSET: f32 = 100.0;
pub const WEAPON_BUTTON_Y_OFFSET: f32 = 200.0;
pub const DOOR_HANDLE_OFFSET: f32 = 5.0;
