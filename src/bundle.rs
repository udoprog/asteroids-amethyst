use amethyst::{
    prelude::SystemExt,
    core::bundle::{Result, SystemBundle},
    ecs::prelude::{DispatcherBuilder},
};
use crate::{
    states::State,
    systems::{
        CollisionSystem, GlobalInputSystem, HandleUiSystem, KillBulletsSystem, LimitObjectsSystem,
        PhysicsSystem, RandomAsteroidSystem, ShipInputSystem,
    },
};

pub struct GlobalBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for GlobalBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(GlobalInputSystem::default(), "global_input", &[]);
        Ok(())
    }
}

pub struct MainBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for MainBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(KillBulletsSystem.pausable(State::Main), "kill_bullets", &[]);
        builder.add(RandomAsteroidSystem::new().pausable(State::Main), "random_asteroids", &[]);
        builder.add(ShipInputSystem.pausable(State::Main), "ship_input_system", &[]);
        builder.add(PhysicsSystem.pausable(State::Main), "physics_system", &[]);
        builder.add(LimitObjectsSystem.pausable(State::Main), "limit_objects", &["physics_system"]);
        builder.add(CollisionSystem.pausable(State::Main), "collisions", &["physics_system"]);
        builder.add(HandleUiSystem.pausable(State::Main), "handle_ui", &[]);
        Ok(())
    }
}
