use bevy::prelude::*;

use crate::components::shape::Shape;
// use bevy_mod_gizmos::*;

pub fn rotate(
    mut query: Query<&mut Transform, With<Shape>>,
    time: Res<Time>
) {
    for mut transform in &mut query {
        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_x(time.delta_seconds() as f32 / 2.0));
        // draw_gizmo(Gizmo::sphere(transform.translation, 0.5, Color::BLUE));
    }
}