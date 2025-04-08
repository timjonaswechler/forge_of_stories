use super::{
    // context::*, // context ist hier nicht direkt nötig, wenn format_* Methoden in Style sind
    ui_link::{LinkStyle, LinkStyleArgs},
    ui_node::{NodeArgs, NodeDataColorStyle, NodeDataLayoutStyle}, // NodeArgs hier importieren
    ui_pin::{PinShape, PinStyle, PinStyleArgs, PinType}, // PinShape/StyleArgs hier importieren
};
use bevy_egui::egui::{self, Color32, Vec2}; // egui importieren für Pos2 etc.

// === ENUMS (ColorStyle, StyleFlags) - Unverändert ===

#[derive(Debug, Clone, Copy)]
pub enum ColorStyle {
    NodeBackground = 0,
    NodeBackgroundHovered,
    NodeBackgroundSelected,
    NodeOutline,
    NodeOutlineActive,
    TitleBar,
    TitleBarHovered,
    TitleBarSelected,
    Link,
    LinkHovered,
    LinkSelected,
    Pin,
    PinHovered,
    BoxSelector,
    BoxSelectorOutline,
    GridBackground,
    GridLine,
    Count,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum StyleFlags {
    None = 0,
    NodeOutline = 1 << 0,
    GridLines = 1 << 2,
}

// === IMPL ColorStyle - Mit der umbenannten Funktion ===

impl ColorStyle {
    // Die Originalen können hier bleiben oder entfernt werden, falls nicht mehr gebraucht
    // pub fn colors_dark() -> ...
    // pub fn colors_classic() -> ...
    // pub fn colors_light() -> ... // Das Original "Light"

    /// Blender "Light" inspired color style (basierend auf der XML) - JETZT KORREKT BENANNT
    pub fn colors_blender_light() -> [egui::Color32; ColorStyle::Count as usize] {
        let mut colors = [egui::Color32::BLACK; ColorStyle::Count as usize];

        colors[ColorStyle::GridBackground as usize] = Color32::from_rgb(0x1d, 0x1d, 0x1d); // #1d1d1d
        colors[ColorStyle::GridLine as usize] = Color32::from_rgb(0x28, 0x28, 0x28); // #282828
        colors[ColorStyle::NodeBackground as usize] = Color32::from_rgb(0x66, 0x66, 0x66); // #666666
        colors[ColorStyle::NodeBackgroundHovered as usize] = Color32::from_gray(0x78); // #787878 (abgeleitet)
        colors[ColorStyle::NodeBackgroundSelected as usize] = Color32::from_rgb(0xed, 0x57, 0x00); // #ed5700
        colors[ColorStyle::TitleBar as usize] = Color32::from_gray(0x5a); // #5a5a5a (abgeleitet)
        colors[ColorStyle::TitleBarHovered as usize] =
            colors[ColorStyle::NodeBackgroundSelected as usize];
        colors[ColorStyle::TitleBarSelected as usize] =
            colors[ColorStyle::NodeBackgroundSelected as usize];
        colors[ColorStyle::NodeOutline as usize] = colors[ColorStyle::GridLine as usize]; // #282828 (abgeleitet)
        colors[ColorStyle::NodeOutlineActive as usize] = Color32::WHITE; // #ffffff
        colors[ColorStyle::Link as usize] = Color32::from_rgb(0x1a, 0x1a, 0x1a); //rgb(117, 117, 117)
        colors[ColorStyle::LinkSelected as usize] =
            Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 0xb3); // #ffffffb3
        colors[ColorStyle::LinkHovered as usize] = colors[ColorStyle::LinkSelected as usize];
        colors[ColorStyle::Pin as usize] = Color32::from_gray(0x96); // #969696 (abgeleitet)
        colors[ColorStyle::PinHovered as usize] = Color32::WHITE; // (abgeleitet)
        let selector_base = colors[ColorStyle::NodeBackgroundSelected as usize];
        colors[ColorStyle::BoxSelector as usize] = selector_base.linear_multiply(0.15);
        colors[ColorStyle::BoxSelectorOutline as usize] = selector_base.linear_multiply(0.5);

        colors
    }
} // Ende impl ColorStyle

// === Struct Style - Unverändert ===
#[derive(Debug)]
pub struct Style {
    pub grid_spacing: f32,
    pub node_corner_rounding: f32,
    pub node_padding_horizontal: f32,
    pub node_padding_vertical: f32,
    pub node_border_thickness: f32,

    pub link_thickness: f32,
    pub link_line_segments_per_length: f32,
    pub link_hover_distance: f32,

    pub pin_shape: PinShape,
    pub pin_circle_radius: f32,
    pub pin_quad_side_length: f32,
    pub pin_triangle_side_length: f32,
    pub pin_line_thickness: f32,
    pub pin_hover_radius: f32,
    pub pin_offset: f32,

    pub flags: usize,
    pub colors: [egui::Color32; ColorStyle::Count as usize],
}

// === IMPL DEFAULT FÜR Style - KORRIGIERT ===
impl Default for Style {
    fn default() -> Self {
        // Ruft die neue Preset-Funktion auf, die *alle* Blender-Style-Werte setzt
        Self::blender_light()
    }
}

// === IMPL Style - MIT NEUER PRESET-FUNKTION ===
impl Style {
    // NEUE Preset-Funktion für den gesamten Blender-Style
    pub fn blender_light() -> Self {
        Self {
            // Parameter aus XML/Annahmen
            grid_spacing: 32.0,           // Standard behalten
            node_corner_rounding: 4.0,    // 0.4 panel_roundness * 10 (Skalierungsfaktor?)
            node_padding_horizontal: 8.0, // Standard behalten
            node_padding_vertical: 8.0,   // Standard behalten
            node_border_thickness: 1.0,   // Standard behalten

            link_thickness: 5.5, // Dünnere Linien wie in Blender 'wire'
            link_line_segments_per_length: 0.1, // Standard behalten (beeinflusst Glätte)
            link_hover_distance: 10.0, // Standard behalten

            pin_shape: PinShape::CircleFilled, // Standard behalten
            pin_circle_radius: 5.0,            // Etwas größer
            pin_quad_side_length: 8.0,
            pin_triangle_side_length: 10.0,
            pin_line_thickness: 1.0,
            pin_hover_radius: 12.0, // Größerer Hover-Bereich
            pin_offset: 0.0,        // Standard behalten

            flags: StyleFlags::NodeOutline as usize | StyleFlags::GridLines as usize, // Standard behalten
            // Farben aus der neuen Blender-Farb-Funktion laden
            colors: ColorStyle::colors_blender_light(),
        }
    }

    // --- Die anderen Methoden bleiben exakt wie zuvor ---
    pub(crate) fn get_screen_space_pin_coordinates(
        &self,
        node_rect: &egui::Rect,
        attribute_rect: &egui::Rect,
        kind: PinType,
    ) -> egui::Pos2 {
        let x = match kind {
            PinType::Input => node_rect.min.x - self.pin_offset,
            _ => node_rect.max.x + self.pin_offset,
        };
        egui::pos2(x, 0.5 * (attribute_rect.min.y + attribute_rect.max.y))
    }

    pub(crate) fn draw_pin_shape(
        &self,
        pin_pos: egui::Pos2,
        pin_shape: PinShape,
        pin_color: egui::Color32,
        shape: egui::layers::ShapeIdx,
        ui: &mut egui::Ui,
    ) {
        let painter = ui.painter();
        match pin_shape {
            PinShape::Circle => painter.set(
                shape,
                egui::Shape::circle_stroke(
                    pin_pos,
                    self.pin_circle_radius,
                    (self.pin_line_thickness, pin_color),
                ),
            ),
            PinShape::CircleFilled => painter.set(
                shape,
                egui::Shape::circle_filled(pin_pos, self.pin_circle_radius, pin_color),
            ),
            PinShape::Quad => painter.set(
                shape,
                egui::Shape::rect_stroke(
                    egui::Rect::from_center_size(
                        pin_pos,
                        [self.pin_quad_side_length / 2.0; 2].into(),
                    ),
                    egui::Rounding::ZERO, // Geändert von CornerRadius::same(0) zu egui::Rounding::ZERO
                    egui::Stroke::new(self.pin_line_thickness, pin_color),
                    egui::StrokeKind::Inside,
                ),
            ),
            PinShape::QuadFilled => painter.set(
                shape,
                egui::Shape::rect_filled(
                    egui::Rect::from_center_size(
                        pin_pos,
                        [self.pin_quad_side_length / 2.0; 2].into(),
                    ),
                    egui::Rounding::ZERO, // Geändert von 0.0 zu egui::Rounding::ZERO
                    pin_color,
                ),
            ),
            PinShape::Triangle => {
                let sqrt_3 = 3f32.sqrt();
                let left_offset = -0.166_666_7 * sqrt_3 * self.pin_triangle_side_length;
                let right_offset = 0.333_333_3 * sqrt_3 * self.pin_triangle_side_length;
                let verticacl_offset = 0.5 * self.pin_triangle_side_length;
                painter.set(
                    shape,
                    egui::Shape::closed_line(
                        vec![
                            pin_pos + egui::vec2(left_offset, verticacl_offset), // Verwende vec2
                            pin_pos + egui::vec2(right_offset, 0.0),
                            pin_pos + egui::vec2(left_offset, -verticacl_offset),
                        ],
                        egui::Stroke::new(self.pin_line_thickness, pin_color), //Stroke hier erstellen
                    ),
                )
            }
            PinShape::TriangleFilled => {
                let sqrt_3 = 3f32.sqrt();
                let left_offset = -0.166_666_7 * sqrt_3 * self.pin_triangle_side_length;
                let right_offset = 0.333_333_3 * sqrt_3 * self.pin_triangle_side_length;
                let verticacl_offset = 0.5 * self.pin_triangle_side_length;
                painter.set(
                    shape,
                    egui::Shape::convex_polygon(
                        vec![
                            pin_pos + egui::vec2(left_offset, verticacl_offset), // Verwende vec2
                            pin_pos + egui::vec2(right_offset, 0.0),
                            pin_pos + egui::vec2(left_offset, -verticacl_offset),
                        ],
                        pin_color,
                        egui::Stroke::NONE,
                    ),
                )
            }
        }
    }

    pub(crate) fn format_node(&self, args: NodeArgs) -> (NodeDataColorStyle, NodeDataLayoutStyle) {
        let mut color = NodeDataColorStyle::default();
        let mut layout = NodeDataLayoutStyle::default();

        color.background = args
            .background
            .unwrap_or(self.colors[ColorStyle::NodeBackground as usize]);
        color.background_hovered = args
            .background_hovered
            .unwrap_or(self.colors[ColorStyle::NodeBackgroundHovered as usize]);
        color.background_selected = args
            .background_selected
            .unwrap_or(self.colors[ColorStyle::NodeBackgroundSelected as usize]);
        color.outline = args
            .outline
            .unwrap_or(self.colors[ColorStyle::NodeOutline as usize]);
        color.titlebar = args
            .titlebar
            .unwrap_or(self.colors[ColorStyle::TitleBar as usize]);
        color.titlebar_hovered = args
            .titlebar_hovered
            .unwrap_or(self.colors[ColorStyle::TitleBarHovered as usize]);
        color.titlebar_selected = args
            .titlebar_selected
            .unwrap_or(self.colors[ColorStyle::TitleBarSelected as usize]);
        layout.corner_rounding = args.corner_rounding.unwrap_or(self.node_corner_rounding);
        layout.padding = args.padding.unwrap_or_else(|| {
            egui::vec2(self.node_padding_horizontal, self.node_padding_vertical)
        });
        layout.border_thickness = args.border_thickness.unwrap_or(self.node_border_thickness);

        (color, layout)
    }

    pub(crate) fn format_pin(&self, args: PinStyleArgs) -> PinStyle {
        PinStyle {
            background: args
                .background
                .unwrap_or(self.colors[ColorStyle::Pin as usize]),
            hovered: args
                .hovered
                .unwrap_or(self.colors[ColorStyle::PinHovered as usize]),
            shape: args.shape.unwrap_or(self.pin_shape),
        }
    }

    pub(crate) fn format_link(&self, args: LinkStyleArgs) -> LinkStyle {
        LinkStyle {
            base: args.base.unwrap_or(self.colors[ColorStyle::Link as usize]),
            hovered: args
                .hovered
                .unwrap_or(self.colors[ColorStyle::LinkHovered as usize]),
            selected: args
                .selected
                .unwrap_or(self.colors[ColorStyle::LinkSelected as usize]),
            thickness: args.thickness.unwrap_or(self.link_thickness),
        }
    }
}
