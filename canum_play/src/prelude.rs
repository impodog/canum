pub use crate::*;
pub use avian2d::prelude::*;
pub use bevy::{ecs::lifecycle::HookContext, prelude::*};
pub use std::time::Duration;

pub use canum_res::{Animation, config::CONFIG};
pub use canum_save::Save;

#[macro_export]
macro_rules! static_system_id {
    ($app: expr, $name: ident, $in: ty, $system: expr) => {
        static $name: std::sync::OnceLock<bevy::ecs::system::SystemId<$in>> =
            std::sync::OnceLock::new();
        $name.set($app.register_system($system)).unwrap();
    };
    ($app: expr, $name: ident, $system: expr) => {
        $crate::static_system_id!($app, $name, (), $system);
    };
}

#[macro_export]
macro_rules! add_observer_hook {
    ($app: expr, $component: ty, $system: expr) => {
        $app.world_mut()
            .register_component_hooks::<$component>()
            .on_add(
                |mut world, bevy::ecs::lifecycle::HookContext { entity, .. }| {
                    world.commands().entity(entity).observe($system);
                },
            );
    };
}
