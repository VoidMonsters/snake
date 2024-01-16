#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct HealthbarMaterial {
    health: f32,
};

@group(1) @binding(0) var<uniform> u: HealthbarMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let lowHealthColor = vec4<f32>(1.0, 0.0, 0.0, 1.0); 
    let highHealthColor = vec4<f32>(0.0, 1.0, 0.0, 1.0);

    if mesh.uv.x > u.health {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    return mix(lowHealthColor, highHealthColor, u.health);
}
