use amethyst::{
    core::bundle::{Result, SystemBundle},
    ecs::prelude::DispatcherBuilder,
};
use crate::systems::{
    CollisionSystem, GlobalInputSystem, HandleUiSystem, KillBulletsSystem, LimitObjectsSystem,
    PhysicsSystem, RandomAsteroidSystem, ShipInputSystem,
};
use crate::enabled::Enabled;
use crate::states::CurrentState;

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
        builder.add(KillBulletsSystem.enabled(CurrentState::Main), "kill_bullets", &[]);
        builder.add(RandomAsteroidSystem::new().enabled(CurrentState::Main), "random_asteroids", &[]);
        builder.add(ShipInputSystem.enabled(CurrentState::Main), "ship_input_system", &[]);
        builder.add(PhysicsSystem.enabled(CurrentState::Main), "physics_system", &[]);
        builder.add(LimitObjectsSystem.enabled(CurrentState::Main), "limit_objects", &["physics_system"]);
        builder.add(CollisionSystem.enabled(CurrentState::Main), "collisions", &["physics_system"]);
        builder.add(HandleUiSystem.enabled(CurrentState::Main), "handle_ui", &[]);
        Ok(())
    }
}
