use super::settings::NodesSettings;
use super::state::GraphUiStateManager;
use super::storage::GraphStorage;
use super::ui_pin::Pin;

use bevy_egui::egui;

pub fn screen_space_to_grid_space(ui_state: &GraphUiStateManager, v: egui::Pos2) -> egui::Pos2 {
    // Von Screen-Koordinaten (relativ zum Fenster/Canvas) zu Grid-Koordinaten (virtueller Raum mit Panning)
    v - ui_state.canvas_origin_screen_space() - ui_state.persistent.panning
}
pub fn grid_space_to_screen_space(ui_state: &GraphUiStateManager, v: egui::Pos2) -> egui::Pos2 {
    // Von Grid-Koordinaten (virtueller Raum mit Panning) zu Screen-Koordinaten (relativ zum Fenster/Canvas)
    v + ui_state.persistent.panning + ui_state.canvas_origin_screen_space()
}
pub fn editor_space_to_screen_space(ui_state: &GraphUiStateManager, v: egui::Pos2) -> egui::Pos2 {
    // Von Editor-Koordinaten (z.B. 0,0 ist oben links im Canvas, ignoriert Panning) zu Screen-Koordinaten
    v + ui_state.canvas_origin_screen_space()
}
pub fn get_screen_space_pin_coordinates(
    ui_state: &GraphUiStateManager, // Zugriff auf Panning/Canvas Origin für Transformation
    storage: &GraphStorage,         // Zugriff auf Nodes für Position
    settings: &NodesSettings,       // Zugriff auf Style für Offset etc.
    pin: &Pin,
) -> egui::Pos2 {
    let Some(parent_node) = storage.get_node(pin.state.parent_node_idx) else {
        // Fallback auf letzte bekannte Screen-Position ODER Grid-Position + Transformation?
        // Sicherer ist vielleicht, hier einen deutlichen Fehler zu signalisieren oder Pos2::ZERO
        return pin.state.pos; // Vorsicht: state.pos ist jetzt im Grid Space! Umwandeln:
                              // return grid_space_to_screen_space(ui_state, pin.state.pos);
    };

    // Node Rect ist im Grid Space
    let node_rect_grid = parent_node.state.rect;

    // Pin Attribut Rect ist relativ zum Node Grid Origin
    let attr_rect_grid_relative = pin.state.attribute_rect;
    let attr_rect_grid_absolute = attr_rect_grid_relative.translate(node_rect_grid.min.to_vec2()); // Addiere Node-Ursprung

    // Wandle die Grid-Rects in Screen-Rects um
    let node_rect_screen = egui::Rect::from_min_max(
        grid_space_to_screen_space(ui_state, node_rect_grid.min),
        grid_space_to_screen_space(ui_state, node_rect_grid.max),
    );
    let attr_rect_screen = egui::Rect::from_min_max(
        grid_space_to_screen_space(ui_state, attr_rect_grid_absolute.min),
        grid_space_to_screen_space(ui_state, attr_rect_grid_absolute.max),
    );

    // Verwende die Style-Methode mit den aktuellen Screen-Rects
    // Stelle sicher, dass die Methode auf settings.style aufgerufen wird
    settings.style.get_screen_space_pin_coordinates(
        &node_rect_screen,
        &attr_rect_screen, // Das ist das Label-Rect im Screen Space
        pin.spec.kind,
    )
}
