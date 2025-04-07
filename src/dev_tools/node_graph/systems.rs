use bevy::color::palettes::basic::*;
use bevy::prelude::*; // Importiere Bevy Preludes // Importiere Bevy Color Preludes
                      // Importiere benötigte Komponenten aus deiner Simulation
                      // Passe diese Pfade entsprechend deiner Struktur an!
use crate::genetics::components::SpeciesGenes;
// use crate::genetics::components::Genotype; // Falls auch Genotyp gebraucht wird

// Importiere die UI-Datenstrukturen und die Ressource
use super::resources::GraphUIData;
use super::ui_data::VisNode;
// use super::ui_data::VisLink; // Für später

/// Dieses System liest Simulationsdaten (Entitäten mit bestimmten Komponenten)
/// und bereitet sie für die Anzeige im Node Graph vor, indem es die `GraphUIData`
/// Ressource füllt.
///
/// Es muss *vor* dem `graph_ui_system` laufen.
pub fn graph_data_provider_system(
    // Query für alle Entitäten, die im Graphen dargestellt werden sollen.
    // Beispiel: Alle Entitäten mit SpeciesGenes. Füge weitere Komponenten hinzu,
    // falls sie für Name, Position oder Links benötigt werden (z.B. Transform).
    entity_query: Query<(Entity, &SpeciesGenes)>,
    // Mutable Zugriff auf die Ressource, die wir füllen wollen.
    mut graph_data: ResMut<GraphUIData>,
) {
    // Lösche die Daten vom letzten Frame
    graph_data.nodes.clear();
    graph_data.links.clear(); // Vorerst auch Links löschen

    let mut current_x = 50.0; // Startposition für einfaches Layout
    const X_SPACING: f32 = 200.0;
    const Y_POS: f32 = 100.0;

    // Iteriere über die gefundenen Entitäten
    for (entity, species) in entity_query.iter() {
        // Erstelle ein VisNode-Objekt für jede Entität
        let node = VisNode {
            // Verwende den Index der Entity als eindeutige ID für die UI
            // ACHTUNG: Entity Indizes sind nicht garantiert stabil über Sessions hinweg!
            // Für persistente Layouts etc. bräuchte man eine stabilere ID.
            id: entity.index() as usize,
            // Speichere die tatsächliche Bevy Entity ID, wichtig für Detailansicht etc.
            entity: Some(entity), // Setze es in ein Some()
            // Erzeuge einen Namen (z.B. aus Spezies)
            name: species.species.join(", "), // Fügt Speziesnamen zusammen
            // Beispiel-Position: Verteile die Nodes horizontal
            position: Vec2::new(current_x, Y_POS), // Verwende Bevy Vec2
            // Standardfarbe (kann später angepasst werden)
            color: Color::from(GRAY),
        };

        // Füge den Node zu den Daten hinzu
        graph_data.nodes.push(node);

        current_x += X_SPACING; // Gehe zur nächsten Position

        // TODO: Hier später Logik hinzufügen, um VisLink-Daten zu erzeugen.
        // Beispiele:
        // - Eltern-Kind-Beziehungen (braucht `Query<&Parent>`)
        // - Interaktions-Partner
        // - Genealogie (wer stammt von wem ab?)
    }
    // info!("Updated GraphUIData: {} nodes", graph_data.nodes.len()); // Optionales Logging
}
