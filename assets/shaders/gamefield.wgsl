#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(1) @binding(0) var<uniform> width: f32;
@group(1) @binding(1) var<uniform> height: f32;
@group(1) @binding(2) var texture: texture_2d<f32>;
@group(1) @binding(3) var textureSampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
        // mesh.uv is a number from 0..1 that we want to convert to a repeating sequence of numbers from 0..1,
        // e.g 0.1 -> 1.0, 0.2 -> 1.0, etc.
    let tex_coords = vec2<f32> (
        (mesh.uv.x % 0.2) * 5.0,
        (mesh.uv.y % 0.4) * 2.5,
    );
    var result = textureSample(texture, textureSampler, tex_coords);

		let border_size = 50.0;

    var alpha_x = 1.0;
    var alpha_y = 1.0;
    let abs_x = mesh.uv.x * width;
    let abs_y = mesh.uv.y * height;

    if abs_x <= border_size {
        alpha_x = abs_x / border_size;
    } else if abs_x >= width - border_size {
        alpha_x = (width-abs_x)/border_size;
    } 
    if abs_y <= border_size {
        alpha_y = abs_y / border_size;
    } else if abs_y >= height - border_size {
        alpha_y = (height-abs_y)/border_size;
    }
    let alpha = (alpha_x + alpha_y) / 2.0;
		if
			mesh.uv.x * width <= border_size || mesh.uv.x * width >= width - border_size ||
			mesh.uv.y * height <= border_size || mesh.uv.y * height >= height - border_size 
		{
      result = mix(result, vec4<f32>(1.0, 0.0, 0.0, 1.0), 1.0 - alpha);
		}

    return result;
}
