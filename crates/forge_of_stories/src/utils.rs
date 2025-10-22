use bevy::{
    ecs::resource::Resource,
    prelude::{Commands, Component, Entity, Query, With},
};

pub fn cleanup<C: Component>(mut commands: Commands, query: Query<Entity, With<C>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
pub fn remove<R: Resource>(mut commands: Commands) {
    commands.remove_resource::<R>();
}
