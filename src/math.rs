use bevy::prelude::*;

pub struct Vector2D {
    angle_in_radians: f32,
    magnitude: f32,
}

impl Vector2D {
    pub fn between(source: Vec3, target: Vec3) -> Self {
        let angle = source.angle_between(target);
        let magnitude = source.distance(target);
        Self {
            angle_in_radians: angle,
            magnitude,
        }
    }
}

// converts an angle/magnitude vector (actual *vector* into a mathematical "vector")
// the Z component will always be zero
// the X component will represent the horizontal magnitude of the vector,
// the Y component will represent the vertical magnitude of the vector
impl Into<Vec3> for Vector2D {
    fn into(self) -> Vec3 {
        Vec3 {
            z: 0.,
            x: self.angle_in_radians.cos() / self.magnitude,
            y: self.angle_in_radians.sin() / self.magnitude,
        }
    }
}
