// src/ui_components/node_graph/resources.rs
use super::ui_data::{VisLink, VisNode};
use bevy::prelude::Resource;

#[derive(Clone, Debug, Default)]
pub struct DetailDisplayData {
    pub title: String,
    pub properties: Vec<(String, String)>,
}
/// Ressource zum Speichern der aufbereiteten Daten für die Node Graph UI.
///
/// Dieses Struct wird vom `graph_data_provider_system` mit Daten aus der Simulation
/// (wie Entitäten, Komponenten, Beziehungen) befüllt.
/// Die UI-Systeme (insbesondere `MyTabViewer::ui`) lesen diese Daten dann,
/// um sie über den `NodesContext` zu visualisieren.
#[derive(Resource, Default, Debug, Clone)]
pub struct GraphUIData {
    pub nodes: Vec<VisNode>,
    pub links: Vec<VisLink>,
    pub selected_node_details_display: Option<DetailDisplayData>,
}
