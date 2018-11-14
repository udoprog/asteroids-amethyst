use amethyst::ecs::prelude::{System, Read};
use amethyst::shred::SystemData;

/// A system that is enabled when the resource `U` has a specific value.
pub struct EnabledSystem<T, U>(T, U);

impl<'s, T, U: 'static> System<'s> for EnabledSystem<T, U>
    where
        T::SystemData: SystemData<'s>,
        T: System<'s>,
        U: Send + Sync + Default + PartialEq<U>,
{
    type SystemData = (Read<'s, U>, T::SystemData);

    fn run(&mut self, data: (Read<'s, U>, T::SystemData)) {
        if self.1 != *data.0 {
            return;
        }

        self.0.run(data.1);
    }
}

pub trait Enabled {
    /// Cause the system to be enabled only when the resource `U` has the specified value.
    fn enabled<U: 'static>(self, state: U) -> EnabledSystem<Self, U>
        where Self: Sized,
              U: Send + Sync + Default + PartialEq<U>;
}

impl<'s, S> Enabled for S
    where S: System<'s>
{
    fn enabled<U: 'static>(self, state: U) -> EnabledSystem<Self, U>
        where Self: Sized,
              U: Send + Sync + Default + PartialEq<U>
    {
        EnabledSystem(self, state)
    }
}
