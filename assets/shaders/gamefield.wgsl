#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(1) @binding(0) var<uniform> width: f32;
@group(1) @binding(1) var<uniform> height: f32;


@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
		var alpha = 0.0;
		let border_size = 3.0;
		if
			mesh.uv.x * width <= border_size || mesh.uv.x * width >= width - border_size ||
			mesh.uv.y * height <= border_size || mesh.uv.y * height >= height - border_size 
		{
			alpha = 1.0;
		} else {
			alpha = log2(min(mesh.uv.x - 0.5, mesh.uv.y - 0.5));
		}

		return vec4<f32>(0.7, 0.0, 0.0, alpha);
}
