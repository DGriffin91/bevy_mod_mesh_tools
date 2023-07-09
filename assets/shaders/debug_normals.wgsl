#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_vertex_output MeshVertexOutput

#import bevy_pbr::pbr_types

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.world_normal, 1.0);
}
