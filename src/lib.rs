use bevy::{
    prelude::*,
    asset::AssetPlugin,
    core_pipeline::{
        bloom::BloomSettings,
        clear_color::ClearColorConfig,
    },
    transform::TransformSystem,
    window::{
        WindowResolution,
        PresentMode,
    },
};

use bevy_asset_loader::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_rapier2d::prelude::*;
use iyes_progress::prelude::*;

mod assets;
mod cix;
mod ext;
mod timed;

pub use assets::*;
pub use cix::*;
pub use timed::*;
pub use ext::*;

pub const PIXELS_PER_METER: f32 = 50.;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    Loading,
    Gameplay,
}

pub fn gameplay_startup_sys(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::NONE),
            },
            ..default()
        },
        BloomSettings {
            intensity: 0.4,
            ..BloomSettings::NATURAL
        },
    ));

    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(32., 8.),
        TransformBundle::from(Transform::from_xyz(0., -160., 0.)),
    ));
}

pub fn run() {
    App::new()
        .add_state::<GameStates>()
        .add_loading_state(LoadingState::new(GameStates::Loading))

        .add_collection_to_loading_state::<_, CixSprites>(GameStates::Loading)
        .init_resource_after_loading_state::<_, GameAtlas>(GameStates::Loading)

        .insert_resource(Msaa::Off)
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0., -9.81 * PIXELS_PER_METER),
            ..default()
        })

        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Cix".into(),
                    resolution: WindowResolution::new(800., 600.),
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            })
            .build()
            .add_before::<AssetPlugin, _>(EmbeddedAssetPlugin)
        )
        .add_plugin(ProgressPlugin::new(GameStates::Loading).continue_to(GameStates::Gameplay))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER))
        //.add_plugin(RapierDebugRenderPlugin::default())

        .add_system(timed_update_sys
            .in_base_set(CoreSet::PreUpdate)
        )
        .add_systems(
            (
                gameplay_startup_sys,
                cix_spawn_sys,
            )
            .in_schedule(OnEnter(GameStates::Gameplay))
        )
        .add_systems(
            (
                cix_update_head_sys,
                cix_spawn_particle_sys.after(cix_update_head_sys),
                cix_update_particle_sys,
                cix_update_fire_sys,
                cix_update_eye_sys,
            )
            .in_set(OnUpdate(GameStates::Gameplay))
        )
        .add_system(cix_spawn_fire_sys
            .in_base_set(CoreSet::PostUpdate)
            .after(TransformSystem::TransformPropagate)
            .run_if(in_state(GameStates::Gameplay))
        )

        .run();
}
