use std::slice::{Iter, IterMut};

use bevy::{
    math::Vec4Swizzles,
    prelude::*,
    render::{
        mesh::{
            skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
            Indices, MeshVertexAttributeId, VertexAttributeValues,
        },
        render_resource::PrimitiveTopology,
    },
};
use thiserror::Error;

#[inline]
pub fn mesh_len(mesh: &Mesh) -> usize {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(positions)) => positions.len(),
        _ => return 0,
    }
}

pub fn mesh_joint_weights(mesh: &Mesh) -> Iter<Vec4> {
    match mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT) {
        Some(VertexAttributeValues::Float32x4(v)) => unsafe {
            std::mem::transmute::<Iter<[f32; 4]>, Iter<Vec4>>(v.iter())
        },
        _ => [].iter(),
    }
}

pub fn mesh_joint_indices(mesh: &Mesh) -> Iter<[u16; 4]> {
    match mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX) {
        Some(VertexAttributeValues::Uint16x4(indices)) => indices.iter(),
        _ => [].iter(),
    }
}

pub fn mesh_positions(mesh: &Mesh) -> Iter<Vec3> {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(v)) => unsafe {
            std::mem::transmute::<Iter<[f32; 3]>, Iter<Vec3>>(v.iter())
        },
        _ => [].iter(),
    }
}

pub fn mesh_positions_mut(mesh: &mut Mesh) -> IterMut<Vec3> {
    match mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(v)) => unsafe {
            std::mem::transmute::<IterMut<[f32; 3]>, IterMut<Vec3>>(v.iter_mut())
        },
        _ => [].iter_mut(),
    }
}

pub fn mesh_normals(mesh: &Mesh) -> Iter<Vec3> {
    match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(v)) => unsafe {
            std::mem::transmute::<Iter<[f32; 3]>, Iter<Vec3>>(v.iter())
        },
        _ => [].iter(),
    }
}

pub fn mesh_normals_mut(mesh: &mut Mesh) -> IterMut<Vec3> {
    match mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(v)) => unsafe {
            std::mem::transmute::<IterMut<[f32; 3]>, IterMut<Vec3>>(v.iter_mut())
        },
        _ => [].iter_mut(),
    }
}

pub fn mesh_uvs(mesh: &Mesh) -> Iter<Vec2> {
    match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(v)) => unsafe {
            std::mem::transmute::<Iter<[f32; 2]>, Iter<Vec2>>(v.iter())
        },
        _ => [].iter(),
    }
}

pub fn mesh_uvs_mut(mesh: &mut Mesh) -> IterMut<Vec2> {
    match mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(v)) => unsafe {
            std::mem::transmute::<IterMut<[f32; 2]>, IterMut<Vec2>>(v.iter_mut())
        },
        _ => [].iter_mut(),
    }
}

pub fn mesh_with_transform(mesh: &Mesh, transform: &Transform) -> Option<Mesh> {
    let mut mesh = mesh.clone();

    let mat = transform.compute_matrix();

    for p in mesh_positions_mut(&mut mesh) {
        *p = mat.transform_point3(*p);
    }

    // Comment below taken from mesh_normal_local_to_world() in mesh_functions.wgsl regarding
    // transform normals from local to world coordinates:

    // NOTE: The mikktspace method of normal mapping requires that the world normal is
    // re-normalized in the vertex shader to match the way mikktspace bakes vertex tangents
    // and normal maps so that the exact inverse process is applied when shading. Blender, Unity,
    // Unreal Engine, Godot, and more all use the mikktspace method. Do not change this code
    // unless you really know what you are doing.
    // http://www.mikktspace.com/

    let inverse_transpose_model = mat.inverse().transpose();
    let inverse_transpose_model = Mat3 {
        x_axis: inverse_transpose_model.x_axis.xyz(),
        y_axis: inverse_transpose_model.y_axis.xyz(),
        z_axis: inverse_transpose_model.z_axis.xyz(),
    };
    for n in mesh_normals_mut(&mut mesh) {
        *n = inverse_transpose_model
            .mul_vec3(*n)
            .normalize_or_zero()
            .into();
    }

    Some(mesh)
}

#[inline]
pub fn skin_model(joint_matrices: &[Mat4], indexes: &[u16; 4], weights: &Vec4) -> Mat4 {
    weights.x * joint_matrices[indexes[0] as usize]
        + weights.y * joint_matrices[indexes[1] as usize]
        + weights.z * joint_matrices[indexes[2] as usize]
        + weights.w * joint_matrices[indexes[3] as usize]
}

#[inline]
pub fn skinned_mesh_joints(
    skin: &SkinnedMesh,
    inverse_bindposes: &Assets<SkinnedMeshInverseBindposes>,
    joints: &Query<&GlobalTransform>,
) -> Option<Vec<Mat4>> {
    let mut buffer = Vec::new();
    let inverse_bindposes = inverse_bindposes.get(&skin.inverse_bindposes)?;

    for (inverse_bindpose, joint) in inverse_bindposes.iter().zip(skin.joints.iter()) {
        if let Ok(joint) = joints.get(*joint) {
            buffer.push(joint.affine() * *inverse_bindpose);
        } else {
            return None;
        }
    }

    Some(buffer)
}

pub fn mesh_with_skinned_transform(
    mesh: &Mesh,
    skinned_mesh: &SkinnedMesh,
    joint_query: &Query<&GlobalTransform>,
    inverse_bindposes: &Assets<SkinnedMeshInverseBindposes>,
) -> Option<Mesh> {
    let mut new_mesh = mesh.clone();

    // get skinned mesh joint models
    if let Some(joints) = skinned_mesh_joints(skinned_mesh, inverse_bindposes, joint_query) {
        let mut models = Vec::with_capacity(mesh_len(&mesh));
        // Use skin model to get world space vertex positions
        for ((pos, indices), weights) in mesh_positions_mut(&mut new_mesh)
            .zip(mesh_joint_indices(mesh))
            .zip(mesh_joint_weights(mesh))
        {
            let model = skin_model(&joints, indices, weights);
            *pos = model.transform_point3(*pos);
            models.push(model);
        }

        // Comment below taken from mesh_normal_local_to_world() in mesh_functions.wgsl regarding
        // transform normals from local to world coordinates:

        // NOTE: The mikktspace method of normal mapping requires that the world normal is
        // re-normalized in the vertex shader to match the way mikktspace bakes vertex tangents
        // and normal maps so that the exact inverse process is applied when shading. Blender, Unity,
        // Unreal Engine, Godot, and more all use the mikktspace method. Do not change this code
        // unless you really know what you are doing.
        // http://www.mikktspace.com/

        for (normal, model) in mesh_normals_mut(&mut new_mesh).zip(models) {
            let inverse_transpose_model = model.inverse().transpose();
            let inverse_transpose_model = Mat3 {
                x_axis: inverse_transpose_model.x_axis.xyz(),
                y_axis: inverse_transpose_model.y_axis.xyz(),
                z_axis: inverse_transpose_model.z_axis.xyz(),
            };
            *normal = inverse_transpose_model
                .mul_vec3(*normal)
                .normalize_or_zero()
                .into();
        }
    }

    Some(new_mesh)
}

#[derive(Error, Debug)]
pub enum MeshAppendError {
    #[error("Attribute {0:?} in destination mesh not found in source mesh.")]
    AttributeNotFound(MeshVertexAttributeId),
}

pub fn mesh_append(dest_mesh: &mut Mesh, src_mesh: &Mesh) -> Result<(), crate::MeshAppendError> {
    let dest_mesh_count = dest_mesh.count_vertices();

    for (attr, _) in dest_mesh.attributes() {
        if src_mesh.attribute(attr).is_none() {
            return Err(MeshAppendError::AttributeNotFound(attr));
        }
    }

    let src_indices = src_mesh.indices().unwrap().iter();

    match dest_mesh.indices_mut().unwrap() {
        bevy::render::mesh::Indices::U16(dv) => {
            for sv in src_indices {
                dv.push(sv as u16 + dest_mesh_count as u16)
            }
        }
        bevy::render::mesh::Indices::U32(dv) => {
            for sv in src_indices {
                dv.push(sv as u32 + dest_mesh_count as u32)
            }
        }
    }

    for (attr, vals) in dest_mesh.attributes_mut() {
        match vals {
            VertexAttributeValues::Float32(v) => {
                if let Some(VertexAttributeValues::Float32(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Sint32(v) => {
                if let Some(VertexAttributeValues::Sint32(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Uint32(v) => {
                if let Some(VertexAttributeValues::Uint32(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Float32x2(v) => {
                if let Some(VertexAttributeValues::Float32x2(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Sint32x2(v) => {
                if let Some(VertexAttributeValues::Sint32x2(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Uint32x2(v) => {
                if let Some(VertexAttributeValues::Uint32x2(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Float32x3(v) => {
                if let Some(VertexAttributeValues::Float32x3(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Sint32x3(v) => {
                if let Some(VertexAttributeValues::Sint32x3(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Uint32x3(v) => {
                if let Some(VertexAttributeValues::Uint32x3(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Float32x4(v) => {
                if let Some(VertexAttributeValues::Float32x4(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Sint32x4(v) => {
                if let Some(VertexAttributeValues::Sint32x4(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Uint32x4(v) => {
                if let Some(VertexAttributeValues::Uint32x4(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Sint16x2(v) => {
                if let Some(VertexAttributeValues::Sint16x2(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Snorm16x2(v) => {
                if let Some(VertexAttributeValues::Snorm16x2(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Uint16x2(v) => {
                if let Some(VertexAttributeValues::Uint16x2(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Unorm16x2(v) => {
                if let Some(VertexAttributeValues::Unorm16x2(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Sint16x4(v) => {
                if let Some(VertexAttributeValues::Sint16x4(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Snorm16x4(v) => {
                if let Some(VertexAttributeValues::Snorm16x4(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Uint16x4(v) => {
                if let Some(VertexAttributeValues::Uint16x4(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Unorm16x4(v) => {
                if let Some(VertexAttributeValues::Unorm16x4(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Sint8x2(v) => {
                if let Some(VertexAttributeValues::Sint8x2(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Snorm8x2(v) => {
                if let Some(VertexAttributeValues::Snorm8x2(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Uint8x2(v) => {
                if let Some(VertexAttributeValues::Uint8x2(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Unorm8x2(v) => {
                if let Some(VertexAttributeValues::Unorm8x2(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Sint8x4(v) => {
                if let Some(VertexAttributeValues::Sint8x4(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Snorm8x4(v) => {
                if let Some(VertexAttributeValues::Snorm8x4(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Uint8x4(v) => {
                if let Some(VertexAttributeValues::Uint8x4(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
            VertexAttributeValues::Unorm8x4(v) => {
                if let Some(VertexAttributeValues::Unorm8x4(s)) = src_mesh.attribute(attr) {
                    v.extend(s);
                }
            }
        }
    }
    Ok(())
}

pub fn mesh_empty_default() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
    mesh.set_indices(Some(Indices::U32(Vec::new())));
    mesh
}

pub fn f32x3_vec3_iter_mut(v: IterMut<[f32; 3]>) -> IterMut<Vec3> {
    unsafe { std::mem::transmute::<IterMut<[f32; 3]>, IterMut<Vec3>>(v) }
}

pub fn f32x4_vec4_iter_mut(v: IterMut<[f32; 4]>) -> IterMut<Vec4> {
    unsafe { std::mem::transmute::<IterMut<[f32; 4]>, IterMut<Vec4>>(v) }
}

pub fn f32x3_vec3_iter(v: Iter<[f32; 3]>) -> Iter<Vec3> {
    unsafe { std::mem::transmute::<Iter<[f32; 3]>, Iter<Vec3>>(v) }
}

pub fn f32x4_vec4_iter(v: Iter<[f32; 4]>) -> Iter<Vec4> {
    unsafe { std::mem::transmute::<Iter<[f32; 4]>, Iter<Vec4>>(v) }
}

pub fn f32x3_vec3_vec_mut(v: &mut Vec<[f32; 3]>) -> &mut Vec<Vec3> {
    unsafe { std::mem::transmute::<&mut Vec<[f32; 3]>, &mut Vec<Vec3>>(v) }
}

pub fn f32x4_vec4_vec_mut(v: &mut Vec<[f32; 4]>) -> &mut Vec<Vec4> {
    unsafe { std::mem::transmute::<&mut Vec<[f32; 4]>, &mut Vec<Vec4>>(v) }
}

pub fn f32x3_vec3_vec(v: &Vec<[f32; 3]>) -> &Vec<Vec3> {
    unsafe { std::mem::transmute::<&Vec<[f32; 3]>, &Vec<Vec3>>(v) }
}

pub fn f32x4_vec4_vec(v: &Vec<[f32; 4]>) -> &Vec<Vec4> {
    unsafe { std::mem::transmute::<&Vec<[f32; 4]>, &Vec<Vec4>>(v) }
}

pub fn f32x3_vec3_iter_mut2(v: IterMut<[f32; 3]>) -> IterMut<Vec3> {
    unsafe { std::mem::transmute::<IterMut<[f32; 3]>, IterMut<Vec3>>(v) }
}
