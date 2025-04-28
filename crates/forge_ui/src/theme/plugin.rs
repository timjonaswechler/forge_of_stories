// crates/forge_ui/src/theme/plugin.rs

use crate::theme::{
    data::*,
    runtime::*,
    // We will move the setup logic into a loading state system
    // systems::{hot_reload_theme_system, setup_theme_resource},
    systems::{hot_reload_theme_system, save_theme_system}, // Keep hot reload system
};
use bevy::{asset::LoadState, prelude::*}; // Import LoadState
use bevy_common_assets::ron::RonAssetPlugin;

// Define a handle resource to track the theme asset
#[derive(Resource)]
struct ThemeAssetHandle(Handle<UiThemeData>);

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<UiThemeData>::new(&["theme.ron"]))
            // --- Asset Loading Logic ---
            // 1. Load the asset during PreStartup and store the handle
            .add_systems(PreStartup, load_theme_asset)
            // 2. Add a system to check loading state *during* your app's loading phase.
            //    Replace 'AppState::Loading' with your actual loading state enum variant.
            //    This system will insert the UiTheme resource once loading is done.
            //    If your app doesn't have a loading state, you might need to add one
            //    or run this check repeatedly in Update until the resource is inserted.
            .add_systems(
                Update,
                check_theme_asset_readiness
                    .run_if(resource_exists::<ThemeAssetHandle>) // Only run if handle exists
                    .run_if(not(resource_exists::<UiTheme>)), // Only run if UiTheme not yet inserted
                                                              // Optional: Run only during a specific loading state
                                                              // .run_if(in_state(AppState::Loading)) // Replace AppState::Loading with your state
            )
            // --- End Asset Loading Logic ---
            .add_systems(Update, hot_reload_theme_system); // Keep hot reload
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            save_theme_system.run_if(bevy::input::common_conditions::input_just_pressed(
                KeyCode::KeyS,
            )),
        );
    }
}

// System to load the asset handle during PreStartup
fn load_theme_asset(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load("theme/default.theme.ron");
    commands.insert_resource(ThemeAssetHandle(handle));
    info!("Initiated loading theme/default.theme.ron");
}

// System to check if the theme asset is loaded and then insert the UiTheme resource
fn check_theme_asset_readiness(
    mut commands: Commands,
    theme_handle_res: Option<Res<ThemeAssetHandle>>, // Make handle optional to avoid panic if removed early
    asset_server: Res<AssetServer>,
    theme_data_assets: Res<Assets<UiThemeData>>,
) {
    // Check if the handle resource exists
    let Some(theme_handle) = theme_handle_res else {
        // Should not happen if run_if condition is correct, but good practice
        // info!("ThemeAssetHandle resource not found."); // Optional log
        return;
    };

    let handle_id = &theme_handle.0;
    let load_state = asset_server.load_state(handle_id);

    // --- DIAGNOSTIC LOG ---
    // Add this log to see the state every time the system runs
    // Be aware: This will be very spammy until loading finishes!
    debug!(
        "Checking theme asset readiness. Handle: {:?}, State: {:?}",
        handle_id.id(),
        load_state
    );
    // ----------------------

    match load_state {
        LoadState::Loaded => {
            info!("Theme asset {:?} reported as Loaded.", handle_id.id()); // Log success
                                                                           // Asset is loaded, proceed with conversion and resource insertion
            if let Some(data) = theme_data_assets.get(handle_id) {
                info!("Successfully retrieved theme data from asset."); // Log data retrieval
                                                                        // --- Conversion Logic (Copied from old setup_theme_resource) ---
                let load_font = |path: &str| -> Handle<Font> {
                    if path.is_empty() {
                        Handle::default()
                    } else {
                        asset_server.load(path)
                    }
                };
                let conv_color = |c: [f32; 4]| Color::srgba(c[0], c[1], c[2], c[3]);

                let typography = UiTypography {
                    /* ... copy from previous theme/systems.rs ... */
                    font_size: UiFontSize {
                        xs: data.font.font_size.xs,
                        sm: data.font.font_size.sm,
                        base: data.font.font_size.base,
                        lg: data.font.font_size.lg,
                        xl: data.font.font_size.xl,
                        x2l: data.font.font_size.x2l,
                        x3l: data.font.font_size.x3l,
                        x4l: data.font.font_size.x4l,
                        x5l: data.font.font_size.x5l,
                        x6l: data.font.font_size.x6l,
                        x7l: data.font.font_size.x7l,
                        x8l: data.font.font_size.x8l,
                        x9l: data.font.font_size.x9l,
                    },
                    font_family: UiFontFamilies {
                        default: load_font(&data.font.font_family.default),
                        sans: FontVariants {
                            light: load_font(&data.font.font_family.sans.light),
                            light_italic: load_font(&data.font.font_family.sans.light_italic),
                            regular: load_font(&data.font.font_family.sans.regular),
                            regular_italic: load_font(&data.font.font_family.sans.regular_italic),
                            medium: load_font(&data.font.font_family.sans.medium),
                            medium_italic: load_font(&data.font.font_family.sans.medium_italic),
                            bold: load_font(&data.font.font_family.sans.bold),
                            bold_italic: load_font(&data.font.font_family.sans.bold_italic),
                        },
                        serif: FontVariants {
                            light: load_font(&data.font.font_family.serif.light),
                            light_italic: load_font(&data.font.font_family.serif.light_italic),
                            regular: load_font(&data.font.font_family.serif.regular),
                            regular_italic: load_font(&data.font.font_family.serif.regular_italic),
                            medium: load_font(&data.font.font_family.serif.medium),
                            medium_italic: load_font(&data.font.font_family.serif.medium_italic),
                            bold: load_font(&data.font.font_family.serif.bold),
                            bold_italic: load_font(&data.font.font_family.serif.bold_italic),
                        },
                        mono: FontVariants {
                            light: load_font(&data.font.font_family.mono.light),
                            light_italic: load_font(&data.font.font_family.mono.light_italic),
                            regular: load_font(&data.font.font_family.mono.regular),
                            regular_italic: load_font(&data.font.font_family.mono.regular_italic),
                            medium: load_font(&data.font.font_family.mono.medium),
                            medium_italic: load_font(&data.font.font_family.mono.medium_italic),
                            bold: load_font(&data.font.font_family.mono.bold),
                            bold_italic: load_font(&data.font.font_family.mono.bold_italic),
                        },
                    },
                };
                let layout = UiLayout {
                    /* ... copy from previous theme/systems.rs ... */
                    spacing: data.layout.spacing,
                    padding: UiSpacing {
                        xs: data.layout.padding.xs,
                        sm: data.layout.padding.sm,
                        base: data.layout.padding.base,
                        lg: data.layout.padding.lg,
                        xl: data.layout.padding.xl,
                        x2l: data.layout.padding.x2l,
                        x3l: data.layout.padding.x3l,
                        x4l: data.layout.padding.x4l,
                        x5l: data.layout.padding.x5l,
                    },
                    margin: UiSpacing {
                        xs: data.layout.margin.xs,
                        sm: data.layout.margin.sm,
                        base: data.layout.margin.base,
                        lg: data.layout.margin.lg,
                        xl: data.layout.margin.xl,
                        x2l: data.layout.margin.x2l,
                        x3l: data.layout.margin.x3l,
                        x4l: data.layout.margin.x4l,
                        x5l: data.layout.margin.x5l,
                    },
                    gap: UiSpacing {
                        xs: data.layout.gap.xs,
                        sm: data.layout.gap.sm,
                        base: data.layout.gap.base,
                        lg: data.layout.gap.lg,
                        xl: data.layout.gap.xl,
                        x2l: data.layout.gap.x2l,
                        x3l: data.layout.gap.x3l,
                        x4l: data.layout.gap.x4l,
                        x5l: data.layout.gap.x5l,
                    },
                    radius: UiRadius {
                        xs: data.layout.radius.xs,
                        sm: data.layout.radius.sm,
                        base: data.layout.radius.base,
                        lg: data.layout.radius.lg,
                        xl: data.layout.radius.xl,
                        x2l: data.layout.radius.x2l,
                        x3l: data.layout.radius.x3l,
                        x4l: data.layout.radius.x4l,
                        full: data.layout.radius.full,
                    },
                };
                let colors = UiColors {
                    /* ... copy from previous theme/systems.rs ... */
                    white: UiColor {
                        background_primary: conv_color(data.color.white.background_primary),
                        background_secondary: conv_color(data.color.white.background_secondary),
                        interaction_primary: conv_color(data.color.white.interaction_primary),
                        interaction_secondary: conv_color(data.color.white.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.white.interaction_tertiary),
                        border_primary: conv_color(data.color.white.border_primary),
                        border_secondary: conv_color(data.color.white.border_secondary),
                        border_tertiary: conv_color(data.color.white.border_tertiary),
                        solid_primary: conv_color(data.color.white.solid_primary),
                        solid_secondary: conv_color(data.color.white.solid_secondary),
                        text_primary: conv_color(data.color.white.text_primary),
                        text_secondary: conv_color(data.color.white.text_secondary),
                    },
                    black: UiColor {
                        background_primary: conv_color(data.color.black.background_primary),
                        background_secondary: conv_color(data.color.black.background_secondary),
                        interaction_primary: conv_color(data.color.black.interaction_primary),
                        interaction_secondary: conv_color(data.color.black.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.black.interaction_tertiary),
                        border_primary: conv_color(data.color.black.border_primary),
                        border_secondary: conv_color(data.color.black.border_secondary),
                        border_tertiary: conv_color(data.color.black.border_tertiary),
                        solid_primary: conv_color(data.color.black.solid_primary),
                        solid_secondary: conv_color(data.color.black.solid_secondary),
                        text_primary: conv_color(data.color.black.text_primary),
                        text_secondary: conv_color(data.color.black.text_secondary),
                    },
                    gray: UiColor {
                        background_primary: conv_color(data.color.gray.background_primary),
                        background_secondary: conv_color(data.color.gray.background_secondary),
                        interaction_primary: conv_color(data.color.gray.interaction_primary),
                        interaction_secondary: conv_color(data.color.gray.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.gray.interaction_tertiary),
                        border_primary: conv_color(data.color.gray.border_primary),
                        border_secondary: conv_color(data.color.gray.border_secondary),
                        border_tertiary: conv_color(data.color.gray.border_tertiary),
                        solid_primary: conv_color(data.color.gray.solid_primary),
                        solid_secondary: conv_color(data.color.gray.solid_secondary),
                        text_primary: conv_color(data.color.gray.text_primary),
                        text_secondary: conv_color(data.color.gray.text_secondary),
                    },
                    mauve: UiColor {
                        background_primary: conv_color(data.color.mauve.background_primary),
                        background_secondary: conv_color(data.color.mauve.background_secondary),
                        interaction_primary: conv_color(data.color.mauve.interaction_primary),
                        interaction_secondary: conv_color(data.color.mauve.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.mauve.interaction_tertiary),
                        border_primary: conv_color(data.color.mauve.border_primary),
                        border_secondary: conv_color(data.color.mauve.border_secondary),
                        border_tertiary: conv_color(data.color.mauve.border_tertiary),
                        solid_primary: conv_color(data.color.mauve.solid_primary),
                        solid_secondary: conv_color(data.color.mauve.solid_secondary),
                        text_primary: conv_color(data.color.mauve.text_primary),
                        text_secondary: conv_color(data.color.mauve.text_secondary),
                    },
                    slate: UiColor {
                        background_primary: conv_color(data.color.slate.background_primary),
                        background_secondary: conv_color(data.color.slate.background_secondary),
                        interaction_primary: conv_color(data.color.slate.interaction_primary),
                        interaction_secondary: conv_color(data.color.slate.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.slate.interaction_tertiary),
                        border_primary: conv_color(data.color.slate.border_primary),
                        border_secondary: conv_color(data.color.slate.border_secondary),
                        border_tertiary: conv_color(data.color.slate.border_tertiary),
                        solid_primary: conv_color(data.color.slate.solid_primary),
                        solid_secondary: conv_color(data.color.slate.solid_secondary),
                        text_primary: conv_color(data.color.slate.text_primary),
                        text_secondary: conv_color(data.color.slate.text_secondary),
                    },
                    sage: UiColor {
                        background_primary: conv_color(data.color.sage.background_primary),
                        background_secondary: conv_color(data.color.sage.background_secondary),
                        interaction_primary: conv_color(data.color.sage.interaction_primary),
                        interaction_secondary: conv_color(data.color.sage.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.sage.interaction_tertiary),
                        border_primary: conv_color(data.color.sage.border_primary),
                        border_secondary: conv_color(data.color.sage.border_secondary),
                        border_tertiary: conv_color(data.color.sage.border_tertiary),
                        solid_primary: conv_color(data.color.sage.solid_primary),
                        solid_secondary: conv_color(data.color.sage.solid_secondary),
                        text_primary: conv_color(data.color.sage.text_primary),
                        text_secondary: conv_color(data.color.sage.text_secondary),
                    },
                    olive: UiColor {
                        background_primary: conv_color(data.color.olive.background_primary),
                        background_secondary: conv_color(data.color.olive.background_secondary),
                        interaction_primary: conv_color(data.color.olive.interaction_primary),
                        interaction_secondary: conv_color(data.color.olive.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.olive.interaction_tertiary),
                        border_primary: conv_color(data.color.olive.border_primary),
                        border_secondary: conv_color(data.color.olive.border_secondary),
                        border_tertiary: conv_color(data.color.olive.border_tertiary),
                        solid_primary: conv_color(data.color.olive.solid_primary),
                        solid_secondary: conv_color(data.color.olive.solid_secondary),
                        text_primary: conv_color(data.color.olive.text_primary),
                        text_secondary: conv_color(data.color.olive.text_secondary),
                    },
                    sand: UiColor {
                        background_primary: conv_color(data.color.sand.background_primary),
                        background_secondary: conv_color(data.color.sand.background_secondary),
                        interaction_primary: conv_color(data.color.sand.interaction_primary),
                        interaction_secondary: conv_color(data.color.sand.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.sand.interaction_tertiary),
                        border_primary: conv_color(data.color.sand.border_primary),
                        border_secondary: conv_color(data.color.sand.border_secondary),
                        border_tertiary: conv_color(data.color.sand.border_tertiary),
                        solid_primary: conv_color(data.color.sand.solid_primary),
                        solid_secondary: conv_color(data.color.sand.solid_secondary),
                        text_primary: conv_color(data.color.sand.text_primary),
                        text_secondary: conv_color(data.color.sand.text_secondary),
                    },
                    tomato: UiColor {
                        background_primary: conv_color(data.color.tomato.background_primary),
                        background_secondary: conv_color(data.color.tomato.background_secondary),
                        interaction_primary: conv_color(data.color.tomato.interaction_primary),
                        interaction_secondary: conv_color(data.color.tomato.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.tomato.interaction_tertiary),
                        border_primary: conv_color(data.color.tomato.border_primary),
                        border_secondary: conv_color(data.color.tomato.border_secondary),
                        border_tertiary: conv_color(data.color.tomato.border_tertiary),
                        solid_primary: conv_color(data.color.tomato.solid_primary),
                        solid_secondary: conv_color(data.color.tomato.solid_secondary),
                        text_primary: conv_color(data.color.tomato.text_primary),
                        text_secondary: conv_color(data.color.tomato.text_secondary),
                    },
                    red: UiColor {
                        background_primary: conv_color(data.color.red.background_primary),
                        background_secondary: conv_color(data.color.red.background_secondary),
                        interaction_primary: conv_color(data.color.red.interaction_primary),
                        interaction_secondary: conv_color(data.color.red.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.red.interaction_tertiary),
                        border_primary: conv_color(data.color.red.border_primary),
                        border_secondary: conv_color(data.color.red.border_secondary),
                        border_tertiary: conv_color(data.color.red.border_tertiary),
                        solid_primary: conv_color(data.color.red.solid_primary),
                        solid_secondary: conv_color(data.color.red.solid_secondary),
                        text_primary: conv_color(data.color.red.text_primary),
                        text_secondary: conv_color(data.color.red.text_secondary),
                    },
                    ruby: UiColor {
                        background_primary: conv_color(data.color.ruby.background_primary),
                        background_secondary: conv_color(data.color.ruby.background_secondary),
                        interaction_primary: conv_color(data.color.ruby.interaction_primary),
                        interaction_secondary: conv_color(data.color.ruby.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.ruby.interaction_tertiary),
                        border_primary: conv_color(data.color.ruby.border_primary),
                        border_secondary: conv_color(data.color.ruby.border_secondary),
                        border_tertiary: conv_color(data.color.ruby.border_tertiary),
                        solid_primary: conv_color(data.color.ruby.solid_primary),
                        solid_secondary: conv_color(data.color.ruby.solid_secondary),
                        text_primary: conv_color(data.color.ruby.text_primary),
                        text_secondary: conv_color(data.color.ruby.text_secondary),
                    },
                    crimson: UiColor {
                        background_primary: conv_color(data.color.crimson.background_primary),
                        background_secondary: conv_color(data.color.crimson.background_secondary),
                        interaction_primary: conv_color(data.color.crimson.interaction_primary),
                        interaction_secondary: conv_color(data.color.crimson.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.crimson.interaction_tertiary),
                        border_primary: conv_color(data.color.crimson.border_primary),
                        border_secondary: conv_color(data.color.crimson.border_secondary),
                        border_tertiary: conv_color(data.color.crimson.border_tertiary),
                        solid_primary: conv_color(data.color.crimson.solid_primary),
                        solid_secondary: conv_color(data.color.crimson.solid_secondary),
                        text_primary: conv_color(data.color.crimson.text_primary),
                        text_secondary: conv_color(data.color.crimson.text_secondary),
                    },
                    pink: UiColor {
                        background_primary: conv_color(data.color.pink.background_primary),
                        background_secondary: conv_color(data.color.pink.background_secondary),
                        interaction_primary: conv_color(data.color.pink.interaction_primary),
                        interaction_secondary: conv_color(data.color.pink.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.pink.interaction_tertiary),
                        border_primary: conv_color(data.color.pink.border_primary),
                        border_secondary: conv_color(data.color.pink.border_secondary),
                        border_tertiary: conv_color(data.color.pink.border_tertiary),
                        solid_primary: conv_color(data.color.pink.solid_primary),
                        solid_secondary: conv_color(data.color.pink.solid_secondary),
                        text_primary: conv_color(data.color.pink.text_primary),
                        text_secondary: conv_color(data.color.pink.text_secondary),
                    },
                    plum: UiColor {
                        background_primary: conv_color(data.color.plum.background_primary),
                        background_secondary: conv_color(data.color.plum.background_secondary),
                        interaction_primary: conv_color(data.color.plum.interaction_primary),
                        interaction_secondary: conv_color(data.color.plum.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.plum.interaction_tertiary),
                        border_primary: conv_color(data.color.plum.border_primary),
                        border_secondary: conv_color(data.color.plum.border_secondary),
                        border_tertiary: conv_color(data.color.plum.border_tertiary),
                        solid_primary: conv_color(data.color.plum.solid_primary),
                        solid_secondary: conv_color(data.color.plum.solid_secondary),
                        text_primary: conv_color(data.color.plum.text_primary),
                        text_secondary: conv_color(data.color.plum.text_secondary),
                    },
                    purple: UiColor {
                        background_primary: conv_color(data.color.purple.background_primary),
                        background_secondary: conv_color(data.color.purple.background_secondary),
                        interaction_primary: conv_color(data.color.purple.interaction_primary),
                        interaction_secondary: conv_color(data.color.purple.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.purple.interaction_tertiary),
                        border_primary: conv_color(data.color.purple.border_primary),
                        border_secondary: conv_color(data.color.purple.border_secondary),
                        border_tertiary: conv_color(data.color.purple.border_tertiary),
                        solid_primary: conv_color(data.color.purple.solid_primary),
                        solid_secondary: conv_color(data.color.purple.solid_secondary),
                        text_primary: conv_color(data.color.purple.text_primary),
                        text_secondary: conv_color(data.color.purple.text_secondary),
                    },
                    violet: UiColor {
                        background_primary: conv_color(data.color.violet.background_primary),
                        background_secondary: conv_color(data.color.violet.background_secondary),
                        interaction_primary: conv_color(data.color.violet.interaction_primary),
                        interaction_secondary: conv_color(data.color.violet.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.violet.interaction_tertiary),
                        border_primary: conv_color(data.color.violet.border_primary),
                        border_secondary: conv_color(data.color.violet.border_secondary),
                        border_tertiary: conv_color(data.color.violet.border_tertiary),
                        solid_primary: conv_color(data.color.violet.solid_primary),
                        solid_secondary: conv_color(data.color.violet.solid_secondary),
                        text_primary: conv_color(data.color.violet.text_primary),
                        text_secondary: conv_color(data.color.violet.text_secondary),
                    },
                    iris: UiColor {
                        background_primary: conv_color(data.color.iris.background_primary),
                        background_secondary: conv_color(data.color.iris.background_secondary),
                        interaction_primary: conv_color(data.color.iris.interaction_primary),
                        interaction_secondary: conv_color(data.color.iris.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.iris.interaction_tertiary),
                        border_primary: conv_color(data.color.iris.border_primary),
                        border_secondary: conv_color(data.color.iris.border_secondary),
                        border_tertiary: conv_color(data.color.iris.border_tertiary),
                        solid_primary: conv_color(data.color.iris.solid_primary),
                        solid_secondary: conv_color(data.color.iris.solid_secondary),
                        text_primary: conv_color(data.color.iris.text_primary),
                        text_secondary: conv_color(data.color.iris.text_secondary),
                    },
                    indigo: UiColor {
                        background_primary: conv_color(data.color.indigo.background_primary),
                        background_secondary: conv_color(data.color.indigo.background_secondary),
                        interaction_primary: conv_color(data.color.indigo.interaction_primary),
                        interaction_secondary: conv_color(data.color.indigo.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.indigo.interaction_tertiary),
                        border_primary: conv_color(data.color.indigo.border_primary),
                        border_secondary: conv_color(data.color.indigo.border_secondary),
                        border_tertiary: conv_color(data.color.indigo.border_tertiary),
                        solid_primary: conv_color(data.color.indigo.solid_primary),
                        solid_secondary: conv_color(data.color.indigo.solid_secondary),
                        text_primary: conv_color(data.color.indigo.text_primary),
                        text_secondary: conv_color(data.color.indigo.text_secondary),
                    },
                    blue: UiColor {
                        background_primary: conv_color(data.color.blue.background_primary),
                        background_secondary: conv_color(data.color.blue.background_secondary),
                        interaction_primary: conv_color(data.color.blue.interaction_primary),
                        interaction_secondary: conv_color(data.color.blue.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.blue.interaction_tertiary),
                        border_primary: conv_color(data.color.blue.border_primary),
                        border_secondary: conv_color(data.color.blue.border_secondary),
                        border_tertiary: conv_color(data.color.blue.border_tertiary),
                        solid_primary: conv_color(data.color.blue.solid_primary),
                        solid_secondary: conv_color(data.color.blue.solid_secondary),
                        text_primary: conv_color(data.color.blue.text_primary),
                        text_secondary: conv_color(data.color.blue.text_secondary),
                    },
                    cyan: UiColor {
                        background_primary: conv_color(data.color.cyan.background_primary),
                        background_secondary: conv_color(data.color.cyan.background_secondary),
                        interaction_primary: conv_color(data.color.cyan.interaction_primary),
                        interaction_secondary: conv_color(data.color.cyan.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.cyan.interaction_tertiary),
                        border_primary: conv_color(data.color.cyan.border_primary),
                        border_secondary: conv_color(data.color.cyan.border_secondary),
                        border_tertiary: conv_color(data.color.cyan.border_tertiary),
                        solid_primary: conv_color(data.color.cyan.solid_primary),
                        solid_secondary: conv_color(data.color.cyan.solid_secondary),
                        text_primary: conv_color(data.color.cyan.text_primary),
                        text_secondary: conv_color(data.color.cyan.text_secondary),
                    },
                    teal: UiColor {
                        background_primary: conv_color(data.color.teal.background_primary),
                        background_secondary: conv_color(data.color.teal.background_secondary),
                        interaction_primary: conv_color(data.color.teal.interaction_primary),
                        interaction_secondary: conv_color(data.color.teal.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.teal.interaction_tertiary),
                        border_primary: conv_color(data.color.teal.border_primary),
                        border_secondary: conv_color(data.color.teal.border_secondary),
                        border_tertiary: conv_color(data.color.teal.border_tertiary),
                        solid_primary: conv_color(data.color.teal.solid_primary),
                        solid_secondary: conv_color(data.color.teal.solid_secondary),
                        text_primary: conv_color(data.color.teal.text_primary),
                        text_secondary: conv_color(data.color.teal.text_secondary),
                    },
                    jade: UiColor {
                        background_primary: conv_color(data.color.jade.background_primary),
                        background_secondary: conv_color(data.color.jade.background_secondary),
                        interaction_primary: conv_color(data.color.jade.interaction_primary),
                        interaction_secondary: conv_color(data.color.jade.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.jade.interaction_tertiary),
                        border_primary: conv_color(data.color.jade.border_primary),
                        border_secondary: conv_color(data.color.jade.border_secondary),
                        border_tertiary: conv_color(data.color.jade.border_tertiary),
                        solid_primary: conv_color(data.color.jade.solid_primary),
                        solid_secondary: conv_color(data.color.jade.solid_secondary),
                        text_primary: conv_color(data.color.jade.text_primary),
                        text_secondary: conv_color(data.color.jade.text_secondary),
                    },
                    green: UiColor {
                        background_primary: conv_color(data.color.green.background_primary),
                        background_secondary: conv_color(data.color.green.background_secondary),
                        interaction_primary: conv_color(data.color.green.interaction_primary),
                        interaction_secondary: conv_color(data.color.green.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.green.interaction_tertiary),
                        border_primary: conv_color(data.color.green.border_primary),
                        border_secondary: conv_color(data.color.green.border_secondary),
                        border_tertiary: conv_color(data.color.green.border_tertiary),
                        solid_primary: conv_color(data.color.green.solid_primary),
                        solid_secondary: conv_color(data.color.green.solid_secondary),
                        text_primary: conv_color(data.color.green.text_primary),
                        text_secondary: conv_color(data.color.green.text_secondary),
                    },
                    grass: UiColor {
                        background_primary: conv_color(data.color.grass.background_primary),
                        background_secondary: conv_color(data.color.grass.background_secondary),
                        interaction_primary: conv_color(data.color.grass.interaction_primary),
                        interaction_secondary: conv_color(data.color.grass.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.grass.interaction_tertiary),
                        border_primary: conv_color(data.color.grass.border_primary),
                        border_secondary: conv_color(data.color.grass.border_secondary),
                        border_tertiary: conv_color(data.color.grass.border_tertiary),
                        solid_primary: conv_color(data.color.grass.solid_primary),
                        solid_secondary: conv_color(data.color.grass.solid_secondary),
                        text_primary: conv_color(data.color.grass.text_primary),
                        text_secondary: conv_color(data.color.grass.text_secondary),
                    },
                    bronze: UiColor {
                        background_primary: conv_color(data.color.bronze.background_primary),
                        background_secondary: conv_color(data.color.bronze.background_secondary),
                        interaction_primary: conv_color(data.color.bronze.interaction_primary),
                        interaction_secondary: conv_color(data.color.bronze.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.bronze.interaction_tertiary),
                        border_primary: conv_color(data.color.bronze.border_primary),
                        border_secondary: conv_color(data.color.bronze.border_secondary),
                        border_tertiary: conv_color(data.color.bronze.border_tertiary),
                        solid_primary: conv_color(data.color.bronze.solid_primary),
                        solid_secondary: conv_color(data.color.bronze.solid_secondary),
                        text_primary: conv_color(data.color.bronze.text_primary),
                        text_secondary: conv_color(data.color.bronze.text_secondary),
                    },
                    gold: UiColor {
                        background_primary: conv_color(data.color.gold.background_primary),
                        background_secondary: conv_color(data.color.gold.background_secondary),
                        interaction_primary: conv_color(data.color.gold.interaction_primary),
                        interaction_secondary: conv_color(data.color.gold.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.gold.interaction_tertiary),
                        border_primary: conv_color(data.color.gold.border_primary),
                        border_secondary: conv_color(data.color.gold.border_secondary),
                        border_tertiary: conv_color(data.color.gold.border_tertiary),
                        solid_primary: conv_color(data.color.gold.solid_primary),
                        solid_secondary: conv_color(data.color.gold.solid_secondary),
                        text_primary: conv_color(data.color.gold.text_primary),
                        text_secondary: conv_color(data.color.gold.text_secondary),
                    },
                    brown: UiColor {
                        background_primary: conv_color(data.color.brown.background_primary),
                        background_secondary: conv_color(data.color.brown.background_secondary),
                        interaction_primary: conv_color(data.color.brown.interaction_primary),
                        interaction_secondary: conv_color(data.color.brown.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.brown.interaction_tertiary),
                        border_primary: conv_color(data.color.brown.border_primary),
                        border_secondary: conv_color(data.color.brown.border_secondary),
                        border_tertiary: conv_color(data.color.brown.border_tertiary),
                        solid_primary: conv_color(data.color.brown.solid_primary),
                        solid_secondary: conv_color(data.color.brown.solid_secondary),
                        text_primary: conv_color(data.color.brown.text_primary),
                        text_secondary: conv_color(data.color.brown.text_secondary),
                    },
                    orange: UiColor {
                        background_primary: conv_color(data.color.orange.background_primary),
                        background_secondary: conv_color(data.color.orange.background_secondary),
                        interaction_primary: conv_color(data.color.orange.interaction_primary),
                        interaction_secondary: conv_color(data.color.orange.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.orange.interaction_tertiary),
                        border_primary: conv_color(data.color.orange.border_primary),
                        border_secondary: conv_color(data.color.orange.border_secondary),
                        border_tertiary: conv_color(data.color.orange.border_tertiary),
                        solid_primary: conv_color(data.color.orange.solid_primary),
                        solid_secondary: conv_color(data.color.orange.solid_secondary),
                        text_primary: conv_color(data.color.orange.text_primary),
                        text_secondary: conv_color(data.color.orange.text_secondary),
                    },
                    amber: UiColor {
                        background_primary: conv_color(data.color.amber.background_primary),
                        background_secondary: conv_color(data.color.amber.background_secondary),
                        interaction_primary: conv_color(data.color.amber.interaction_primary),
                        interaction_secondary: conv_color(data.color.amber.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.amber.interaction_tertiary),
                        border_primary: conv_color(data.color.amber.border_primary),
                        border_secondary: conv_color(data.color.amber.border_secondary),
                        border_tertiary: conv_color(data.color.amber.border_tertiary),
                        solid_primary: conv_color(data.color.amber.solid_primary),
                        solid_secondary: conv_color(data.color.amber.solid_secondary),
                        text_primary: conv_color(data.color.amber.text_primary),
                        text_secondary: conv_color(data.color.amber.text_secondary),
                    },
                    yellow: UiColor {
                        background_primary: conv_color(data.color.yellow.background_primary),
                        background_secondary: conv_color(data.color.yellow.background_secondary),
                        interaction_primary: conv_color(data.color.yellow.interaction_primary),
                        interaction_secondary: conv_color(data.color.yellow.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.yellow.interaction_tertiary),
                        border_primary: conv_color(data.color.yellow.border_primary),
                        border_secondary: conv_color(data.color.yellow.border_secondary),
                        border_tertiary: conv_color(data.color.yellow.border_tertiary),
                        solid_primary: conv_color(data.color.yellow.solid_primary),
                        solid_secondary: conv_color(data.color.yellow.solid_secondary),
                        text_primary: conv_color(data.color.yellow.text_primary),
                        text_secondary: conv_color(data.color.yellow.text_secondary),
                    },
                    lime: UiColor {
                        background_primary: conv_color(data.color.lime.background_primary),
                        background_secondary: conv_color(data.color.lime.background_secondary),
                        interaction_primary: conv_color(data.color.lime.interaction_primary),
                        interaction_secondary: conv_color(data.color.lime.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.lime.interaction_tertiary),
                        border_primary: conv_color(data.color.lime.border_primary),
                        border_secondary: conv_color(data.color.lime.border_secondary),
                        border_tertiary: conv_color(data.color.lime.border_tertiary),
                        solid_primary: conv_color(data.color.lime.solid_primary),
                        solid_secondary: conv_color(data.color.lime.solid_secondary),
                        text_primary: conv_color(data.color.lime.text_primary),
                        text_secondary: conv_color(data.color.lime.text_secondary),
                    },
                    mint: UiColor {
                        background_primary: conv_color(data.color.mint.background_primary),
                        background_secondary: conv_color(data.color.mint.background_secondary),
                        interaction_primary: conv_color(data.color.mint.interaction_primary),
                        interaction_secondary: conv_color(data.color.mint.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.mint.interaction_tertiary),
                        border_primary: conv_color(data.color.mint.border_primary),
                        border_secondary: conv_color(data.color.mint.border_secondary),
                        border_tertiary: conv_color(data.color.mint.border_tertiary),
                        solid_primary: conv_color(data.color.mint.solid_primary),
                        solid_secondary: conv_color(data.color.mint.solid_secondary),
                        text_primary: conv_color(data.color.mint.text_primary),
                        text_secondary: conv_color(data.color.mint.text_secondary),
                    },
                    sky: UiColor {
                        background_primary: conv_color(data.color.sky.background_primary),
                        background_secondary: conv_color(data.color.sky.background_secondary),
                        interaction_primary: conv_color(data.color.sky.interaction_primary),
                        interaction_secondary: conv_color(data.color.sky.interaction_secondary),
                        interaction_tertiary: conv_color(data.color.sky.interaction_tertiary),
                        border_primary: conv_color(data.color.sky.border_primary),
                        border_secondary: conv_color(data.color.sky.border_secondary),
                        border_tertiary: conv_color(data.color.sky.border_tertiary),
                        solid_primary: conv_color(data.color.sky.solid_primary),
                        solid_secondary: conv_color(data.color.sky.solid_secondary),
                        text_primary: conv_color(data.color.sky.text_primary),
                        text_secondary: conv_color(data.color.sky.text_secondary),
                    },
                };
                // --- End Conversion Logic ---

                let theme = UiTheme {
                    ui_scaling: data.ui_scaling,
                    font: typography,
                    layout,
                    color: colors,
                };

                commands.insert_resource(theme);
                // Remove the handle *after* inserting the resource
                // Use remove_resource::<ThemeAssetHandle>() directly on commands
                commands.remove_resource::<ThemeAssetHandle>();
                info!("UiTheme resource inserted and ThemeAssetHandle removed.");

                // Optional: Transition to the next app state if needed
                // commands.insert_resource(NextState(Some(AppState::MainMenu)));
            } else {
                // This case should ideally not happen if LoadState::Loaded is true,
                // but handle it just in case.
                error!("UiThemeData asset data not found for handle {:?} even though LoadState is Loaded. Asset might be empty or invalid.", handle_id.id());
            }
        }
        LoadState::Failed(error) => {
            // Log the failure and the error details
            error!(
                "Failed to load theme asset {:?}: {:?}",
                handle_id.id(),
                error // Log the specific asset error
            );
            // Consider removing the handle so we don't check it anymore
            commands.remove_resource::<ThemeAssetHandle>();
        }
        LoadState::Loading => {
            // Still loading, do nothing - the debug log above shows this state
            info!("Theme asset {:?} is still loading.", handle_id.id());
        }
        LoadState::NotLoaded => {
            // Asset might not have been processed by asset server yet
            debug!("Theme asset {:?} is NotLoaded.", handle_id.id());
        }
    }
    // If still loading or not started, do nothing and wait for the next run
}
