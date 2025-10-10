// Compare normals between using mesh_with_transform vs bevy's gpu implementation

use bevy::{
    prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, shader::ShaderRef,
};
use bevy_mod_mesh_tools::{mesh_append, mesh_with_transform};

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut debug_material: ResMut<Assets<DebugNormalsMaterial>>,
) {
    let cube_mesh = Cuboid::default().mesh().into();
    let sphere_mesh = Sphere::default().mesh().uv(32, 18);

    let mut mesh_a = mesh_with_transform(&cube_mesh, &Transform::from_xyz(-2.0, 0.0, 0.0)).unwrap();
    let mesh_b = mesh_with_transform(&sphere_mesh, &Transform::from_xyz(2.0, 0.0, 0.0)).unwrap();

    mesh_append(&mut mesh_a, &mesh_b).unwrap();

    commands.spawn((
        Mesh3d(meshes.add(mesh_a.clone())),
        MeshMaterial3d(debug_material.add(DebugNormalsMaterial {})),
        Transform::from_xyz(0.0, 3., 0.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(mesh_a.clone())),
        MeshMaterial3d(debug_material.add(DebugNormalsMaterial {})),
        MeshUpdate {
            transform: Transform::IDENTITY,
            mesh: mesh_a,
        },
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));
}

#[derive(Component)]
struct MeshUpdate {
    transform: Transform,
    mesh: Mesh,
}

fn rotate(
    time: Res<Time>,
    mut mesh_update: Query<(&Mesh3d, &mut MeshUpdate)>,
    mut rotate_only: Query<&mut Transform, (With<Mesh3d>, Without<MeshUpdate>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let r1 = 1.2 * time.delta_secs();
    let r2 = 1.1 * time.delta_secs();
    for (mesh_h, mut m) in &mut mesh_update {
        m.transform.rotate_x(r1);
        m.transform.rotate_z(r2);
        if let Some(mesh) = meshes.get_mut(&*mesh_h) {
            *mesh = mesh_with_transform(&m.mesh, &m.transform).unwrap();
        }
    }
    for mut trans in &mut rotate_only {
        trans.rotate_x(r1);
        trans.rotate_z(r2);
    }
}

impl Material for DebugNormalsMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/debug_normals.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct DebugNormalsMaterial {}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MaterialPlugin::<DebugNormalsMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate)
        .run();
}
