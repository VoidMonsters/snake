// This shader draws a circle with a given input color
#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var<uniform> color: vec4<f32>;
@group(1) @binding(1) var<uniform> highlight: u32;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // the UVs are now adjusted around the middle of the rect.
    let uv = in.uv * 2.0 - 1.0;
    
    var alpha = 0.0;
    if highlight != u32(0) {
        // circle alpha, the higher the power the harsher the falloff.
        alpha = 1.0 - pow(sqrt(dot(uv, uv)), 2.0);
    }

    return vec4<f32>(color.rgb, alpha);
}
