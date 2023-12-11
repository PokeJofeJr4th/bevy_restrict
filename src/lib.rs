//! # Bevy Restrict
//! Utilities for restricting the use of certain bevy features
use std::marker::PhantomData;

use bevy::{
    ecs::{
        query::ReadOnlyWorldQuery,
        system::{EntityCommands, SystemParam},
    },
    prelude::*,
};

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use super::{
        entity_cleanup_system, marker_components, resource_cleanup_system, spawn_button,
        spawn_default_system, square_sprite, state_resource_plugin_from_world,
        state_resource_plugin_given, ButtonStyle, ClosurePlugin, EntityDespawner, EntitySpawner,
        ResourceHandle, SquareSprite,
    };
}

#[macro_export]
macro_rules! marker_components {
    ($($(# $tt:tt)*$id:ident),*) => {
        $(
            #[derive(Clone, Copy, Default, ::bevy::ecs::component::Component, Hash, PartialEq, Eq, PartialOrd, Ord)]
            $(# $tt)*
            pub struct $id;
        )*
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SquareSprite {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub color: Color,
    pub size: f32,
    pub grid: f32,
}

impl Default for SquareSprite {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            color: Color::BLACK,
            size: 100.0,
            grid: 100.0,
        }
    }
}

pub fn square_sprite(sprite: SquareSprite) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            color: sprite.color,
            custom_size: Some(Vec2::new(sprite.size, sprite.size)),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(sprite.x * sprite.grid, sprite.y * sprite.grid, sprite.z),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn entity_cleanup_system<C: Component, Q: ReadOnlyWorldQuery>(
    mut despawner: EntityDespawner,
    query: Query<Entity, (With<C>, Q)>,
) {
    query.for_each(|ent| {
        // println!("Entity Cleanup System: Despawning {ent:?}");
        despawner.despawn_recursive(ent);
    });
}

pub fn resource_cleanup_system<R: Resource>(mut resource: ResourceHandle<R>) {
    resource.remove();
}

pub struct ClosurePlugin<T: Fn(&mut App) + Send + Sync + 'static>(T);

impl<T: Fn(&mut App) + Send + Sync + 'static> Plugin for ClosurePlugin<T> {
    fn build(&self, app: &mut App) {
        self.0(app);
    }
}

pub fn state_resource_plugin_given<S: States + Clone, R: Resource + Clone>(
    state: S,
    resource: R,
) -> impl Plugin {
    let insert_resource_system = move |mut handle: ResourceHandle<R>| {
        handle.insert(resource.clone());
    };
    ClosurePlugin(move |app: &mut App| {
        app.add_systems(OnEnter(state.clone()), insert_resource_system.clone())
            .add_systems(OnExit(state.clone()), resource_cleanup_system::<R>);
    })
}

pub fn state_resource_plugin_from_world<S: States + Clone, R: Resource + FromWorld>(
    state: S,
) -> impl Plugin {
    let insert_resource_system = |mut resource: ResourceHandle<R>| {
        resource.init();
    };
    ClosurePlugin(move |app| {
        app.add_systems(OnEnter(state.clone()), insert_resource_system)
            .add_systems(OnExit(state.clone()), resource_cleanup_system::<R>);
    })
}

#[derive(SystemParam)]
pub struct EntitySpawner<'w, 's, C: Bundle>(Commands<'w, 's>, PhantomData<C>);

impl<'w, 's, 'a, C: Bundle + Default> EntitySpawner<'w, 's, C> {
    pub fn spawn_default_with(&'a mut self, bundle: impl Bundle) -> EntityCommands<'w, 's, 'a> {
        self.0.spawn((C::default(), bundle))
    }

    pub fn spawn_default(&'a mut self) -> EntityCommands<'w, 's, 'a> {
        self.0.spawn(C::default())
    }
}

pub fn spawn_default_system<C: Bundle + Default>(mut spawner: EntitySpawner<C>) {
    spawner.spawn_default();
}

pub struct ButtonStyle {
    pub width: Val,
    pub height: Val,
    pub background_color: Color,
    pub font_size: f32,
    pub text_color: Color,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            width: Val::Px(150.0),
            height: Val::Px(65.0),
            background_color: Color::DARK_GRAY,
            font_size: 28.0,
            text_color: Color::WHITE,
        }
    }
}

pub fn spawn_button<B: Component + Default>(
    parent: &mut ChildBuilder,
    text: impl Into<String>,
    style: ButtonStyle,
) {
    parent
        .spawn((
            B::default(),
            ButtonBundle {
                style: Style {
                    width: style.width,
                    height: style.height,
                    margin: UiRect::all(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(5.0)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: BackgroundColor(style.background_color),
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font_size: style.font_size,
                    color: style.text_color,
                    ..Default::default()
                },
            ));
        });
}

impl<'w, 's, 'a, C: Bundle> EntitySpawner<'w, 's, C> {
    pub fn spawn_with(&'a mut self, entity: C, bundle: impl Bundle) -> EntityCommands<'w, 's, 'a> {
        self.0.spawn((entity, bundle))
    }

    pub fn spawn(&'a mut self, entity: C) -> EntityCommands<'w, 's, 'a> {
        self.0.spawn(entity)
    }
}

#[derive(SystemParam)]
pub struct EntityDespawner<'w, 's>(Commands<'w, 's>);

impl<'w, 's, 'a> EntityDespawner<'w, 's> {
    pub fn despawn(&'a mut self, entity: Entity) {
        // println!("EntityDespawner: Despawning {entity:?}");
        self.0.entity(entity).despawn();
    }

    pub fn despawn_recursive(&'a mut self, entity: Entity) {
        // println!("EntityDespawner: Recursively Despawning {entity:?}");
        self.0.entity(entity).despawn_recursive();
    }
}

#[derive(SystemParam)]
pub struct ResourceHandle<'w, 's, R: Resource>(Commands<'w, 's>, PhantomData<R>);

impl<'w, 's, 'a, R: Resource> ResourceHandle<'w, 's, R> {
    pub fn remove(&'a mut self) {
        self.0.remove_resource::<R>();
    }

    pub fn init(&'a mut self)
    where
        R: FromWorld,
    {
        self.0.init_resource::<R>();
    }

    pub fn insert(&'a mut self, resource: R) {
        self.0.insert_resource(resource);
    }
}
