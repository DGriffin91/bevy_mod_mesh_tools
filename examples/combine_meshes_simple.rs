// Combine multiple meshes with Transforms

use bevy::prelude::*;
use bevy_mod_mesh_tools::{mesh_append, mesh_with_transform};

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let cube_mesh = Cuboid::default().mesh();
    let sphere_mesh = Sphere::default().mesh().uv(32, 18);

    let mut mesh_a = mesh_with_transform(&cube_mesh, &Transform::from_xyz(-2.0, 0.0, 0.0)).unwrap();
    let mesh_b = mesh_with_transform(&sphere_mesh, &Transform::from_xyz(2.0, 0.0, 0.0)).unwrap();

    mesh_append(&mut mesh_a, &mesh_b).unwrap();

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh_a),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}
