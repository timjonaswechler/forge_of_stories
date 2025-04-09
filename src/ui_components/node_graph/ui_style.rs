use super::{
    // context::*, // context ist hier nicht direkt nötig, wenn format_* Methoden in Style sind
    ui_link::{LinkStyle, LinkStyleArgs},
    ui_node::{NodeArgs, NodeDataColorStyle, NodeDataLayoutStyle}, // NodeArgs hier importieren
    ui_pin::{PinShape, PinStyle, PinStyleArgs, PinType}, // PinShape/StyleArgs hier importieren
};
use bevy::log::*;
use bevy_egui::egui::{self, Color32}; // egui importieren für Pos2 etc.
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

        // Hintergrund und Grid
        colors[ColorStyle::GridBackground as usize] = Color32::from_rgb(0x1d, 0x1d, 0x1d); // Dunkelgrau
        colors[ColorStyle::GridLine as usize] = Color32::from_rgb(0x28, 0x28, 0x28); // Grid-Linien

        // Node Farben
        colors[ColorStyle::NodeBackground as usize] = Color32::from_rgb(0x66, 0x66, 0x66); // Node BG
        colors[ColorStyle::NodeBackgroundHovered as usize] = Color32::from_rgb(0x78, 0x78, 0x78);
        colors[ColorStyle::NodeBackgroundSelected as usize] = Color32::from_rgb(0xed, 0x57, 0x00); // Orange

        // Titelleiste
        colors[ColorStyle::TitleBar as usize] = Color32::from_rgb(0x5a, 0x5a, 0x5a); // Etwas dunkler
        colors[ColorStyle::TitleBarHovered as usize] = Color32::from_rgb(0xed, 0x57, 0x00);
        colors[ColorStyle::TitleBarSelected as usize] = Color32::from_rgb(0xed, 0x57, 0x00);

        // Rahmen
        colors[ColorStyle::NodeOutline as usize] = Color32::from_rgb(0x28, 0x28, 0x28); // Dunkler Rahmen
        colors[ColorStyle::NodeOutlineActive as usize] = Color32::from_rgb(0xff, 0xff, 0xff);

        // Links/Verbindungen
        colors[ColorStyle::Link as usize] = Color32::from_rgb(0x1a, 0x1a, 0x1a); // Link normal
        colors[ColorStyle::LinkSelected as usize] =
            Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 0xb3);
        colors[ColorStyle::LinkHovered as usize] =
            Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 0xb3);

        // Pins
        colors[ColorStyle::Pin as usize] = Color32::from_rgb(0x96, 0x96, 0x96); // Standard
        colors[ColorStyle::PinHovered as usize] = Color32::from_rgb(0xff, 0xff, 0xff); // Weiß beim Hover

        // Box Selection
        colors[ColorStyle::BoxSelector as usize] =
            Color32::from_rgba_unmultiplied(0xed, 0x57, 0x00, 0x15);
        colors[ColorStyle::BoxSelectorOutline as usize] =
            Color32::from_rgba_unmultiplied(0xed, 0x57, 0x00, 0x50);

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
            // Grid-Einstellungen
            grid_spacing: 32.0,
            // Node-Layout
            node_corner_rounding: 4.0, // Reduziert für Blender-Look
            node_padding_horizontal: 8.0,
            node_padding_vertical: 8.0,
            node_border_thickness: 1.0, // Dünnerer Rahmen wie in Blender

            // Link-Appearance
            link_thickness: 3.0, // Etwas dünner für Blender-Look
            link_line_segments_per_length: 0.1,
            link_hover_distance: 6.0,

            // Pin-Einstellungen beibehalten aber anpassen
            pin_shape: PinShape::CircleFilled,
            pin_circle_radius: 4.0, // Etwas kleiner
            pin_quad_side_length: 8.0,
            pin_triangle_side_length: 10.0,
            pin_line_thickness: 1.0,
            pin_hover_radius: 10.0,
            pin_offset: 0.0,

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
        pin_pos: egui::Pos2,               // Die Position des Pin-Mittelpunkts
        pin_shape: PinShape, // Welche Form (jetzt nur noch Circle/CircleFilled relevant)
        pin_color: egui::Color32, // Die (Füll-)Farbe des Pins
        shape_idx: egui::layers::ShapeIdx, // Der Index zum Setzen des Shapes im Painter
        ui: &mut egui::Ui,   // Das UI zum Holen des Painters
    ) {
        let painter = ui.painter();
        // Definiere den Outline Stroke (1 Pixel breit, schwarz)
        let outline_stroke = egui::Stroke::new(1.0, Color32::BLACK);

        // Da wir nur Kreise wollen, unterscheiden wir nur, ob gefüllt oder nicht.
        // Wir verwenden die `shape` Eigenschaft des PinStyle, die in `format_pin`
        // entweder auf den per `PinStyleArgs` gesetzten Wert oder den Default (CircleFilled) gesetzt wird.
        match pin_shape {
            PinShape::CircleFilled => {
                // Zeichne einen GEFÜLLTEN Kreis mit schwarzer Outline
                painter.set(
                    shape_idx,
                    egui::Shape::Circle(egui::epaint::CircleShape {
                        center: pin_pos,
                        radius: self.pin_circle_radius, // Radius aus dem Style
                        fill: pin_color,                // Die übergebene Pin-Farbe als Füllung
                        stroke: outline_stroke,         // Der definierte schwarze Stroke
                    }),
                );
            }
            PinShape::Circle => {
                // Zeichne nur den UMRISS eines Kreises (Outline ist hier die Hauptfarbe)
                // Verwende pin_color für den Stroke, aber füge auch eine dünne schwarze Außenlinie hinzu,
                // damit auch ungefüllte Kreise eine konsistente Outline haben.
                // Zeichne zuerst die schwarze Outline (etwas größer)
                painter.set(
                    shape_idx, // Benutze denselben Index, der letzte set() gewinnt oder sie überlagern
                    egui::Shape::Circle(egui::epaint::CircleShape {
                        center: pin_pos,
                        radius: self.pin_circle_radius + outline_stroke.width / 2.0, // Leicht größer für außen
                        fill: Color32::TRANSPARENT,
                        stroke: outline_stroke,
                    }),
                );
                // Zeichne den farbigen Stroke darüber (mit Originalgröße)
                // Braucht eigentlich einen neuen ShapeIndex oder komplexeres Zeichnen.
                // Einfacher: Nur die farbige Linie zeichnen und auf Outline verzichten bei `Circle`.
                /* // Alternative nur farbiger Stroke:
                 painter.set(
                    shape_idx,
                    egui::Shape::circle_stroke(
                         pin_pos,
                         self.pin_circle_radius,
                         egui::Stroke::new(self.pin_line_thickness, pin_color) // Standard dicke Linie
                    ),
                );
                 */
                // Kompromiss: Nur den gefüllten Kreis verwenden, Circle ignorieren.
                // Kopiere den Code von CircleFilled hierher, um sicherzustellen, dass immer
                // etwas Sichtbares mit Outline gezeichnet wird.
                painter.set(
                    shape_idx,
                    egui::Shape::Circle(egui::epaint::CircleShape {
                        center: pin_pos,
                        radius: self.pin_circle_radius,
                        fill: pin_color, // Behandle Circle wie CircleFilled
                        stroke: outline_stroke,
                    }),
                );
            }
            // Ignoriere alle anderen Shapes (Triangle, Quad etc.) komplett
            _ => {
                // Zeichne nichts oder einen Fallback (z.B. einen kleinen schwarzen Punkt)
                painter.set(
                    shape_idx,
                    egui::Shape::circle_filled(pin_pos, 1.0, Color32::BLACK), // Mini-Punkt als Fallback
                );
                warn!(
                    "Unsupported PinShape {:?} encountered in draw_pin_shape. Drawing fallback.",
                    pin_shape
                );
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
