use bevy::prelude::*;
use bevy_hanabi::prelude::*;

// Yoinked from: https://github.com/djeedai/bevy_hanabi
pub fn setup(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    // Define a color gradient from red to transparent black
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.4, 0.4, 0.4, 1.));
    gradient.add_key(1.0, Vec4::splat(0.0));

    // Create a new expression module
    let mut module = Module::default();

    // On spawn, randomly initialize the position of the particle
    // to be over the surface of a sphere of radius 2 units.
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(0.05),
        dimension: ShapeDimension::Surface,
    };

    // Also initialize a radial initial velocity to 6 units/sec
    // away from the (same) sphere center.
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(Vec3::Y * 0.1),
    };

    // Initialize the total lifetime of the particle, that is
    // the time for which it's simulated and rendered. This modifier
    // is almost always required, otherwise the particles won't show.
    let lifetime = module.lit(1.0); // literal value "10.0"
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Every frame, add a gravity-like acceleration downward
    let accel = module.lit(Vec3::new(0.0, 0.1, 0.0));
    let update_accel = AccelModifier::new(accel);

    let texture_slot = module.lit(0);

    // Create the effect asset
    let effect = EffectAsset::new(
        // Maximum number of particles alive at a time
        1,
        // Spawn at a rate of 5 particles per second
        Spawner::rate(100.0.into()),
        // Move the expression module into the asset
        module,
    )
    .with_name("BulletHit")
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .update(update_accel)
    // Render the particles with a color gradient over their
    // lifetime. This maps the gradient key 0 to the particle spawn
    // time, and the gradient key 1 to the particle death (10s).
    .render(ColorOverLifetimeModifier { gradient })
    .render(ParticleTextureModifier {
        texture_slot,
        sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
    })
    .render(OrientModifier {
        mode: OrientMode::FaceCameraPosition,
        rotation: None,
    })
    .render(SizeOverLifetimeModifier {
        gradient: Gradient::constant([0.2; 3].into()),
        screen_space_size: false,
    });

    // Insert into the asset system
    effects.add(effect)
}
