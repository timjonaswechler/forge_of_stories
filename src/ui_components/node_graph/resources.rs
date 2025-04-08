// src/ui_components/node_graph/resources.rs

use bevy::prelude::Resource;
// Importiere deine UI-Datenstrukturen aus dem ui_data Modul
use super::ui_data::{VisLink, VisNode}; // Pfad anpassen, falls ui_data woanders liegt

/// Ressource zum Speichern der aufbereiteten Daten für die Node Graph UI.
///
/// Dieses Struct wird vom `graph_data_provider_system` mit Daten aus der Simulation
/// (wie Entitäten, Komponenten, Beziehungen) befüllt.
/// Die UI-Systeme (insbesondere `MyTabViewer::ui`) lesen diese Daten dann,
/// um sie über den `NodesContext` zu visualisieren.
#[derive(Resource, Default, Debug, Clone)] // Clone hinzugefügt für Übergabe in `show`
pub struct GraphUIData {
    // Vektor mit allen Knoten (Nodes), die im Graphen angezeigt werden sollen.
    pub nodes: Vec<VisNode>,

    // Vektor mit allen Verbindungen (Links/Edges) zwischen den Pins der Knoten.
    pub links: Vec<VisLink>,
    // Optional könnten hier auch global gesammelte Pin-Daten stehen,
    // falls sie nicht direkt Teil von VisNode sind.
    // pub pins: Vec<VisPin>,
}
