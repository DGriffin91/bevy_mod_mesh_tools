//! Skinned mesh example with mesh and joints data defined in code.
//! Example taken from <https://github.com/KhronosGroup/glTF-Tutorials/blob/master/gltfTutorial/gltfTutorial_019_SimpleSkin.md>

use std::f32::consts::PI;

use bevy::{
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    render::{
        mesh::{
            skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
            Indices, MeshAabb, PrimitiveTopology, VertexAttributeValues,
        },
        render_asset::RenderAssetUsages,
    },
};
use bevy_mod_mesh_tools::{mesh_positions, mesh_with_skinned_transform};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, WireframePlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (joint_animation, skinned_vertex_locations))
        .run();
}

/// Used to mark a joint to be animated in the [`joint_animation`] system.
#[derive(Component)]
struct AnimatedJoint;

/// Construct a mesh and a skeleton with 2 joints for that mesh,
///   and mark the second joint to be animated.
/// It is similar to the scene defined in `models/SimpleSkin/SimpleSkin.gltf`
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut skinned_mesh_inverse_bindposes_assets: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    // Create a camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Create inverse bindpose matrices for a skeleton consists of 2 joints
    let inverse_bindposes =
        skinned_mesh_inverse_bindposes_assets.add(SkinnedMeshInverseBindposes::from(vec![
            Mat4::from_translation(Vec3::new(-0.5, -1.0, 0.0)),
            Mat4::from_translation(Vec3::new(-0.5, -1.0, 0.0)),
        ]));

    // Create a mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    // Set mesh vertex positions
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.5, 0.0],
            [1.0, 0.5, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.5, 0.0],
            [1.0, 1.5, 0.0],
            [0.0, 2.0, 0.0],
            [1.0, 2.0, 0.0],
        ],
    );
    // Set mesh vertex normals
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 10]);
    // Set mesh vertex UVs. Although the mesh doesn't have any texture applied,
    //  UVs are still required by the render pipeline. So these UVs are zeroed out.
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 10]);
    // Set mesh vertex joint indices for mesh skinning.
    // Each vertex gets 4 indices used to address the `JointTransforms` array in the vertex shader
    //  as well as `SkinnedMeshJoint` array in the `SkinnedMesh` component.
    // This means that a maximum of 4 joints can affect a single vertex.
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        // Need to be explicit here as [u16; 4] could be either Uint16x4 or Unorm16x4.
        VertexAttributeValues::Uint16x4(vec![
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 1, 0, 0],
            [0, 1, 0, 0],
            [0, 1, 0, 0],
            [0, 1, 0, 0],
            [0, 1, 0, 0],
            [0, 1, 0, 0],
            [0, 1, 0, 0],
            [0, 1, 0, 0],
        ]),
    );
    // Set mesh vertex joint weights for mesh skinning.
    // Each vertex gets 4 joint weights corresponding to the 4 joint indices assigned to it.
    // The sum of these weights should equal to 1.
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_WEIGHT,
        vec![
            [1.00, 0.00, 0.0, 0.0],
            [1.00, 0.00, 0.0, 0.0],
            [0.75, 0.25, 0.0, 0.0],
            [0.75, 0.25, 0.0, 0.0],
            [0.50, 0.50, 0.0, 0.0],
            [0.50, 0.50, 0.0, 0.0],
            [0.25, 0.75, 0.0, 0.0],
            [0.25, 0.75, 0.0, 0.0],
            [0.00, 1.00, 0.0, 0.0],
            [0.00, 1.00, 0.0, 0.0],
        ],
    );
    // Tell bevy to construct triangles from a list of vertex indices,
    //  where each 3 vertex indices form an triangle.
    mesh.insert_indices(Indices::U16(vec![
        0, 1, 3, 0, 3, 2, 2, 3, 5, 2, 5, 4, 4, 5, 7, 4, 7, 6, 6, 7, 9, 6, 9, 8,
    ]));

    let mesh_h = meshes.add(mesh);

    // Create joint entities
    let joint_0 = commands.spawn(Transform::default()).id();
    let joint_1 = commands.spawn((AnimatedJoint, Transform::IDENTITY)).id();

    // Set joint_1 as a child of joint_0.
    commands.entity(joint_0).add_children(&[joint_1]);

    // Each joint in this vector corresponds to each inverse bindpose matrix in `SkinnedMeshInverseBindposes`.
    let joint_entities = vec![joint_0, joint_1];

    // Create skinned mesh renderer. Note that its transform doesn't affect the position of the mesh.
    commands
        .spawn((
            Mesh3d(mesh_h.clone()),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
        ))
        .insert(SkinnedMesh {
            inverse_bindposes: inverse_bindposes.clone(),
            joints: joint_entities,
        });

    // debug cubes for each vertex
    for _ in 0..10 {
        commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.1))),
                MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 1.0))),
            ))
            .insert(DebugVertex);
    }

    // AABB debug cube
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::default())),
            MeshMaterial3d(materials.add(Color::srgba(0.0, 0.0, 0.0, 0.0))),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ))
        .insert(
            // This enables wireframe drawing on this entity
            Wireframe,
        )
        .insert(AABBDebugCube);
}

#[derive(Component)]
struct DebugVertex;

#[derive(Component)]
struct AABBDebugCube;

/// Animate the joint marked with [`AnimatedJoint`] component.
fn joint_animation(time: Res<Time>, mut query: Query<&mut Transform, With<AnimatedJoint>>) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_axis_angle(Vec3::Z, 0.5 * PI * time.elapsed_secs().sin());
    }
}

fn skinned_vertex_locations(
    query: Query<(&Mesh3d, &SkinnedMesh)>,
    meshes: Res<Assets<Mesh>>,
    inverse_bindposes: Res<Assets<SkinnedMeshInverseBindposes>>,
    joint_query: Query<&GlobalTransform>,
    mut debug_vertex_cubes: Query<&mut Transform, (With<DebugVertex>, Without<AABBDebugCube>)>,
    mut aabb_debug_cube: Query<&mut Transform, (With<AABBDebugCube>, Without<DebugVertex>)>,
) {
    for (mesh_h, skinned_mesh) in query.iter() {
        if let Some(mesh) = meshes.get(mesh_h) {
            let ws_mesh =
                mesh_with_skinned_transform(&mesh, skinned_mesh, &joint_query, &inverse_bindposes)
                    .unwrap();

            // update debug cube positions to match world space vertices
            for (mut trans, ws_pos) in debug_vertex_cubes.iter_mut().zip(mesh_positions(&ws_mesh)) {
                trans.translation = *ws_pos;
            }

            let ws_aabb = ws_mesh.compute_aabb().unwrap();
            //update aabb debug cube
            if let Some(mut trans) = aabb_debug_cube.iter_mut().next() {
                trans.translation = ws_aabb.center.into();
                trans.scale = (ws_aabb.half_extents * 2.0).into();
            }
        }
    }
}
