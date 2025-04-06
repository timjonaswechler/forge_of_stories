// src/simulation/systems/event_handlers.rs
use crate::attributes::components::{
    AttributeGroup, MentalAttributes, PhysicalAttributes, SocialAttributes,
};
use crate::genetics::{Phenotype, SpeciesGenes};
use crate::simulation::events::{EntityInitializedEvent, TemporaryAttributeModifierEvent};
use bevy::prelude::*;

// System zum Senden von EntityInitializedEvents
pub fn send_entity_initialized_events(
    query: Query<(Entity, &SpeciesGenes), Added<Phenotype>>,
    mut entity_initialized_events: EventWriter<EntityInitializedEvent>,
) {
    for (entity, species_genes) in query.iter() {
        entity_initialized_events.send(EntityInitializedEvent {
            entity,
            species: species_genes.species.clone(),
        });
        info!(
            "Entity {:?} wurde initialisiert (Phenotype hinzugefügt, Spezies: {:?})",
            entity, species_genes.species
        );
    }
}

// handle_temporary_attribute_modifiers
pub fn handle_temporary_attribute_modifiers(
    _commands: Commands,
    _time: Res<Time>,
    mut temp_modifier_events: EventReader<TemporaryAttributeModifierEvent>,
    mut query: Query<(
        &mut PhysicalAttributes,
        &mut MentalAttributes,
        &mut SocialAttributes,
    )>,
) {
    for event in temp_modifier_events.read() {
        if let Ok((mut physical, mut mental, mut social)) = query.get_mut(event.entity) {
            let attribute_ref_option = physical
                .get_attribute_mut(event.attribute_id)
                .or_else(|| mental.get_attribute_mut(event.attribute_id))
                .or_else(|| social.get_attribute_mut(event.attribute_id));

            if let Some(attribute) = attribute_ref_option {
                let old_value = attribute.current_value;
                attribute.current_value += event.value_change;
                attribute.current_value = attribute.current_value.clamp(0.0, attribute.max_value);

                info!(
                    "TempMod angewendet auf Entität {:?}: Attribut '{}' ({:?}) geändert von {:.1} um {:+.1} -> Neuer Wert: {:.1} (Dauer: {:.1}s)",
                    event.entity,
                    attribute.name,
                    event.attribute_id,
                    old_value,
                    event.value_change,
                    attribute.current_value,
                    event.duration
                );
            } else {
                warn!(
                    "Attribut Enum '{:?}' für temporären Modifikator auf {:?} konnte in keiner Attributgruppe gefunden werden.",
                    event.attribute_id, event.entity
                );
            }
        } else {
            warn!("Entität {:?} für TempMod nicht gefunden.", event.entity);
        }
    }
}
