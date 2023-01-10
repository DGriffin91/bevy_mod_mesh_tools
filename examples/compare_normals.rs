// Compare normals between using mesh_with_transform vs bevy's gpu implementation

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_mod_mesh_tools::{mesh_append, mesh_with_transform};

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut debug_material: ResMut<Assets<DebugNormalsMaterial>>,
) {
    let cube_mesh = shape::Cube::default().into();
    let sphere_mesh = shape::UVSphere::default().into();

    let mut mesh_a = mesh_with_transform(&cube_mesh, &Transform::from_xyz(-2.0, 0.0, 0.0)).unwrap();
    let mesh_b = mesh_with_transform(&sphere_mesh, &Transform::from_xyz(2.0, 0.0, 0.0)).unwrap();

    mesh_append(&mut mesh_a, &mesh_b).unwrap();

    commands.spawn((MaterialMeshBundle {
        mesh: meshes.add(mesh_a.clone()),
        material: debug_material.add(DebugNormalsMaterial {}),
        transform: Transform::from_xyz(0.0, 3., 0.0),
        ..default()
    },));

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(mesh_a.clone()),
            material: debug_material.add(DebugNormalsMaterial {}),
            ..default()
        },
        MeshUpdate {
            transform: Transform::IDENTITY,
            mesh: mesh_a,
        },
    ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });
}

#[derive(Component)]
struct MeshUpdate {
    transform: Transform,
    mesh: Mesh,
}

fn rotate(
    time: Res<Time>,
    mut mesh_update: Query<(&Handle<Mesh>, &mut MeshUpdate)>,
    mut rotate_only: Query<&mut Transform, (With<Handle<Mesh>>, Without<MeshUpdate>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let r1 = 1.2 * time.delta_seconds();
    let r2 = 1.1 * time.delta_seconds();
    for (mesh_h, mut m) in &mut mesh_update {
        m.transform.rotate_x(r1);
        m.transform.rotate_z(r2);
        if let Some(mesh) = meshes.get_mut(&mesh_h) {
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

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "717f64fe-6844-4821-8926-e0ed374294c9"]
pub struct DebugNormalsMaterial {}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<DebugNormalsMaterial>::default())
        .add_startup_system(setup)
        .add_system(rotate)
        .run();
}
