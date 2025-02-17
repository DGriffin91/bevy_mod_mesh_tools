// Combine multiple meshes with Transforms

use bevy::prelude::*;
use bevy_mod_mesh_tools::{mesh_append, mesh_with_transform};

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = Cuboid::default().mesh().into();
    let sphere_mesh = Sphere::default().mesh().uv(32, 18);

    let mut mesh_a = mesh_with_transform(&cube_mesh, &Transform::from_xyz(-2.0, 0.0, 0.0)).unwrap();
    let mesh_b = mesh_with_transform(&sphere_mesh, &Transform::from_xyz(2.0, 0.0, 0.0)).unwrap();

    mesh_append(&mut mesh_a, &mesh_b).unwrap();

    commands.spawn((
        Mesh3d(meshes.add(mesh_a)),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}
