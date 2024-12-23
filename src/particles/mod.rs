use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    hierarchy::DespawnRecursiveExt,
    math::Vec3,
    time::Time,
    transform::components::Transform,
};
use bevy_hanabi::{EffectAsset, EffectMaterial, ParticleEffect, ParticleEffectBundle};

mod bullet_hit;
mod demo;

#[allow(unused)]
#[derive(Resource)]
pub struct ParticleMap {
    pub demo: Handle<EffectAsset>,
    pub bullet_hit: Handle<EffectAsset>,
}

impl ParticleMap {
    #[allow(unused)]
    pub fn spawn_demo(&self, commands: &mut Commands, pos: Vec3) {
        commands
            .spawn(ParticleEffectBundle {
                effect: ParticleEffect::new(self.demo.clone_weak()),
                transform: Transform::from_translation(pos),
                ..Default::default()
            })
            .insert(ParticleLifetime::new(2.0));
    }

    pub fn spawn_bullet_hit(&self, asset_server: &AssetServer, commands: &mut Commands, pos: Vec3) {
        let texture_handle = asset_server.load("particles/bullethit.png");
        commands
            .spawn(ParticleEffectBundle {
                effect: ParticleEffect::new(self.bullet_hit.clone_weak()),
                transform: Transform::from_translation(pos),
                ..Default::default()
            })
            .insert(ParticleLifetime::new(1.0))
            .insert(EffectMaterial {
                images: vec![texture_handle],
            });
    }
}

#[derive(Component)]
pub struct ParticleLifetime {
    time: f32,
}
impl ParticleLifetime {
    pub fn new(time: f32) -> Self {
        Self { time }
    }
    pub fn update(
        mut commands: Commands,
        mut particles: Query<(Entity, &mut ParticleLifetime)>,
        time: Res<Time>,
    ) {
        let diff = time.delta_secs();
        for (ent, mut pl) in &mut particles {
            pl.time -= diff;
            if pl.time <= 0.0 {
                commands.entity(ent).despawn_recursive();
            }
        }
    }
}

pub fn register_particles(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    _asset_server: Res<AssetServer>,
) {
    let map = ParticleMap {
        demo: demo::setup(&mut effects),
        bullet_hit: bullet_hit::setup(&mut effects),
    };

    commands.insert_resource(map);
}
