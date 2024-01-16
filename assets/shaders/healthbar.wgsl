// Assuming health is a float between 0.0 and 1.0
// Red to green gradient for low to high health

[[block]]
struct Uniforms {
    health: f32;
};

[[group(0), binding(0)]]
var<uniform> u: Uniforms;

[[stage(fragment)]]
fn main() -> [[location(0)]] vec4<f32> {
    // Define your gradient colors
    let lowHealthColor = vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red
    let highHealthColor = vec4<f32>(0.0, 1.0, 0.0, 1.0); // Green

    // Interpolate between colors based on health
    let barColor = mix(lowHealthColor, highHealthColor, u.health);

    return barColor;
}
