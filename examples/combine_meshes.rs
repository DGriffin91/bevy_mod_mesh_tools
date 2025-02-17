// Combine multiple meshes with Transforms
// Adapted from https://github.com/bevyengine/bevy/blob/main/examples/3d/3d_shapes.rs

use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bevy_mod_mesh_tools::{mesh_append, mesh_empty_default, mesh_with_transform};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .run();
}

const X_EXTENT: f32 = 14.;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let shapes = [
        Cuboid::default().mesh().into(),
        Capsule3d::default().mesh().into(),
        Torus::default().mesh().into(),
        Cylinder::default().mesh().into(),
        Sphere::default().mesh().ico(5).unwrap(),
        Sphere::default().mesh().uv(32, 18),
    ];

    let mut combined_mesh = mesh_empty_default();

    for (i, shape) in shapes.iter().enumerate() {
        let trans = Transform::from_xyz(
            -X_EXTENT / 2. + i as f32 / (shapes.len() - 1) as f32 * X_EXTENT,
            2.0,
            0.0,
        )
        .with_rotation(Quat::from_rotation_x(-PI / 4.));
        let mesh = mesh_with_transform(shape, &trans).unwrap();
        mesh_append(&mut combined_mesh, &mesh).unwrap();
    }

    commands.spawn((
        Mesh3d(meshes.add(combined_mesh)),
        MeshMaterial3d(debug_material),
    ));

    commands.spawn((
        PointLight {
            intensity: 9000.0 * 1000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
