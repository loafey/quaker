use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    hierarchy::DespawnRecursiveExt,
    time::Time,
};

#[derive(Debug, Component)]
pub struct Message {
    time: f32,
}
impl Default for Message {
    fn default() -> Self {
        Self { time: 6.0 }
    }
}
impl Message {
    pub fn update_messages(
        mut commands: Commands,
        mut query: Query<(Entity, &mut Message)>,
        time: Res<Time>,
    ) {
        for (ent, mut mesg) in &mut query {
            mesg.time -= time.delta_seconds();
            if mesg.time <= 0.0 {
                commands.entity(ent).despawn_recursive();
            }
        }
    }
}
