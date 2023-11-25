use bevy::{prelude::{
    Color, Commands, Deref, DirectionalLight, DirectionalLightBundle, Entity, ParamSet, Plugin,
    Query, Res, Resource, Transform, Vec3, With, Startup, Update,
}, ecs::{schedule::{OnEnter, IntoSystemConfigs, common_conditions::in_state}, component::Component, system::ResMut}, time::{Timer, TimerMode, Time}, math::Quat, asset::Assets, render::mesh::{Mesh, shape}, pbr::{StandardMaterial, PbrBundle}};
use bevy_atmosphere::{model::AtmosphereModel, system_param::AtmosphereMut, collection::nishita::Nishita};

use crate::AppState;

use super::player::PlayerController;

// #[derive(Resource, Deref)]
// pub struct SkyLightEntity(Entity);

// fn setup_environment(mut cmds: Commands) {
//     const _SIZE: f32 = 200.0; //make this dynamic according to view distance???

//     let sky_light_entity = cmds
//         .spawn(DirectionalLightBundle {
//             transform: Transform::IDENTITY.looking_to(Vec3::new(-1.0, -0.6, -1.0), Vec3::Y),
//             directional_light: DirectionalLight {
//                 color: Color::WHITE,
//                 shadows_enabled: true,
//                 // shadow_projection: OrthographicProjection {
//                 //     // left: -SIZE,
//                 //     // right: SIZE,
//                 //     // bottom: -SIZE,
//                 //     // top: SIZE,
//                 //     near: -SIZE,
//                 //     far: SIZE,
//                 //     ..Default::default()
//                 // },
//                 ..Default::default()
//             },
//             ..Default::default()
//         })
//         .id();

//     cmds.insert_resource(SkyLightEntity(sky_light_entity));
// }

// fn daylight_cycle(
//     sky_light_entity: Res<SkyLightEntity>,
//     mut queries: ParamSet<(
//         Query<&mut Transform>,
//         Query<&Transform, With<PlayerController>>,
//     )>,
// ) {
//     let sky_light_entity = **sky_light_entity;
//     let player_translation = queries
//         .p1()
//         .get_single()
//         .map_or_else(|_| Default::default(), |ply| ply.translation);

//     {
//         let mut binding = queries.p0();
//         let mut sky_light_transform = binding.get_mut(sky_light_entity).unwrap();
//         sky_light_transform.translation = player_translation;
//     }
// }

fn setup_environment(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::WHITE,
                shadows_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        Sun,
    ));

    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0})),
    //     material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.8, 0.8))),
    //     ..Default::default()
    // });
}

fn daylight_cycle(
    mut atmosphere: AtmosphereMut<Nishita>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<CycleTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let t = time.elapsed_seconds_wrapped() / 2.0;
        atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

        if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
            light_trans.rotation = Quat::from_rotation_x(-t);
            directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
        }
    }
}

#[derive(Component)]
struct Sun;

#[derive(Resource)]
struct CycleTimer(Timer);


pub struct VoxelWorldSkyboxPlugin;

impl Plugin for VoxelWorldSkyboxPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .insert_resource(AtmosphereModel::default())
            .insert_resource(CycleTimer(Timer::new(
                bevy::utils::Duration::from_millis(50),
                TimerMode::Repeating,
            )))
            .add_systems(OnEnter(AppState::InGame), setup_environment)
            .add_systems(Update, (daylight_cycle).run_if(in_state(AppState::InGame)));
    }
}