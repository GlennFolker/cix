#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(let_chains)]

#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use bevy::{
    prelude::*,
    asset::AssetPlugin,
    core_pipeline::clear_color::ClearColor,
    render::camera::CameraUpdateSystem,
    window::{
        WindowResolution,
        PresentMode,
    },
};

use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_rapier2d::prelude::*;
use iyes_progress::prelude::*;
use leafwing_input_manager::prelude::*;

pub mod ext;

mod assets;
mod camera;
mod collide;
mod enemies;
mod health;
mod cix;
mod timed;
mod world;

pub use assets::*;
pub use camera::*;
pub use collide::*;
pub use enemies::*;
pub use health::*;
pub use cix::*;
pub use timed::*;
pub use world::*;

pub const PIXELS_PER_METER: f32 = 100.;

pub const GROUP_CIX: Group = Group::GROUP_1;
pub const GROUP_ENEMY: Group = Group::GROUP_2;
pub const GROUP_STATIC: Group = Group::GROUP_3;
pub const GROUP_BULLET: Group = Group::GROUP_30;
pub const GROUP_STOP_PIERCE: Group = Group::GROUP_31;
pub const GROUP_GROUND: Group = Group::GROUP_32;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameStates {
    #[default]
    Loading,
    Gameplay,
}

fn main() {
    App::new()
        .add_state::<GameStates>()
        .add_state::<CixStates>()
        .add_loading_state(LoadingState::new(GameStates::Loading))

        .add_collection_to_loading_state::<_, LdtkWorld>(GameStates::Loading)
        .add_collection_to_loading_state::<_, BackgroundImages>(GameStates::Loading)
        .add_collection_to_loading_state::<_, GenericSprites>(GameStates::Loading)
        .add_collection_to_loading_state::<_, CixSprites>(GameStates::Loading)
        .add_collection_to_loading_state::<_, StaticEnemySprites>(GameStates::Loading)
        .init_resource_after_loading_state::<_, GameAtlas>(GameStates::Loading)

        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(Msaa::Off)
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0., -9.81 * PIXELS_PER_METER),
            timestep_mode: TimestepMode::Variable {
                max_dt: 1.0 / 20.0,
                time_scale: 1.0,
                substeps: 1,
            },
            ..default()
        })

        .insert_resource(LdtkSettings {
            set_clear_color: SetClearColor::No,
            int_grid_rendering: IntGridRendering::Invisible,
            level_background: LevelBackground::Nonexistent,
            ..default()
        })
        .insert_resource(LevelSelection::Identifier("trauma".into()))

        .insert_resource(CameraPos(Vec2::splat(0.)))
        .insert_resource(CixSpawnPos(Vec2::splat(0.)))

        .insert_resource(EnemyGears::default())

        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_linear())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Cix".into(),
                    resolution: WindowResolution::new(800., 500.),
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            })
            .build()
            .add_before::<AssetPlugin, _>(EmbeddedAssetPlugin)
        )

        .add_plugin(ProgressPlugin::new(GameStates::Loading).continue_to(GameStates::Gameplay))
        .add_plugin(InputManagerPlugin::<CixAction>::default())
        .add_plugin(LdtkPlugin)
        .add_plugin(RapierPhysicsPlugin::<()>::pixels_per_meter(PIXELS_PER_METER))
        .add_plugin(RapierDebugRenderPlugin {
            //enabled: cfg!(debug_assertions),
            enabled: false,
            ..default()
        })

        .add_event::<DeathEvent>()

        .add_startup_systems((
            camera_spawn_sys,
            camera_viewport_sys
                .in_base_set(StartupSet::PostStartup)
                .before(CameraUpdateSystem),
        ))
        .add_system(camera_viewport_sys
            .in_base_set(CoreSet::PostUpdate)
            .before(CameraUpdateSystem)
        )

        .add_systems((timed_update_sys, health_update_sys).in_base_set(CoreSet::PreUpdate))
        .add_systems((timed_post_update_sys, health_post_update_sys).in_base_set(CoreSet::PostUpdate))

        .add_systems((world_start_sys, world_fade_add_sys).in_schedule(OnEnter(GameStates::Gameplay)))
        .add_system(world_post_start_sys
            .in_base_set(CoreSet::PreUpdate)
            .run_if(in_state(GameStates::Gameplay))
        )
        .add_systems((
            world_fade_update_sys,
            world_update_bg_sys,
            collide_sys,
        ).in_set(OnUpdate(GameStates::Gameplay)))
        .add_system(world_start_update_sys
            .run_if(in_state(GameStates::Gameplay))
            .run_if(in_state(CixStates::Nonexistent))
        )

        .add_systems((cix_pre_update_sys, cix_follow_camera_sys)
            .in_base_set(CoreSet::PreUpdate)
            .distributive_run_if(in_state(CixStates::Alive))
        )

        .add_system(cix_init_spawn_sys.in_schedule(OnEnter(CixStates::Spawning)))
        .add_system(cix_update_spawn_sys.in_set(OnUpdate(CixStates::Spawning)))
        .add_systems((
            cix_update_fire_sys,
            cix_update_particle_sys,
            cix_attack_update_sys,
            cix_update_death_sys,
        ).in_set(OnUpdate(GameStates::Gameplay)))
        .add_systems((
            cix_check_alive_sys,
            cix_update_sys,
            cix_update_head_sys,
            cix_spawn_particle_sys.after(cix_update_head_sys),
            cix_flip_direction_sys,
            cix_update_direction_sys.after(cix_flip_direction_sys),
            cix_direct_attire_sys.after(cix_update_direction_sys),
            cix_update_arm_sys,
            cix_spawn_fire_sys,
            cix_update_eye_sys,
        ).in_set(OnUpdate(CixStates::Alive)))
        .add_systems((
            cix_move_sys,
            cix_jump_sys,
            cix_attack_input_sys,
            cix_attack_sys.after(cix_attack_input_sys),
        ).in_set(OnUpdate(CixStates::Alive)))
        .add_system(cix_respawn_sys.in_set(OnUpdate(CixStates::Dead)))

        .add_system(enemy_gear_init_sys
            .in_base_set(CoreSet::PreUpdate)
            .run_if(in_state(GameStates::Gameplay))
        )
        .add_systems((
            enemy_gear_update_sys,
            enemy_barrier_update_sys, enemy_barrier_particle_update_sys
        ).in_set(OnUpdate(GameStates::Gameplay)))

        .run();
}
