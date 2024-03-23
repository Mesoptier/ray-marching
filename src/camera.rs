use std::f32::consts::FRAC_PI_2;

pub(crate) struct Camera {
    pub target: [f32; 3],
    pub position: [f32; 3],
}

pub(crate) enum OrbitCameraControllerEvent {
    Pan([f32; 2]),
    Orbit([f32; 2]),
    // Zoom(f32),
}

pub(crate) struct OrbitCameraController {
    /// Target to look at.
    target: [f32; 3],
    /// Pitch angle in radians.
    pitch: f32,
    /// Yaw angle in radians.
    yaw: f32,
    /// Distance from the target.
    distance: f32,

    yaw_speed: f32,
    pitch_speed: f32,
}

impl OrbitCameraController {
    pub(crate) fn new(target: [f32; 3], distance: f32) -> Self {
        Self {
            target,
            pitch: 0.0,
            yaw: 0.0,
            distance,

            yaw_speed: 0.01,
            pitch_speed: 0.01,
        }
    }

    pub(crate) fn camera(&self) -> Camera {
        let position = [
            self.target[0] + self.yaw.cos() * self.pitch.cos() * self.distance,
            self.target[1] + self.pitch.sin() * self.distance,
            self.target[2] + self.yaw.sin() * self.pitch.cos() * self.distance,
        ];
        Camera {
            target: self.target,
            position,
        }
    }

    pub(crate) fn update(&mut self, event: OrbitCameraControllerEvent) {
        match event {
            OrbitCameraControllerEvent::Pan([dx, dy]) => {
                let forward = [
                    self.yaw.cos() * self.pitch.cos(),
                    self.pitch.sin(),
                    self.yaw.sin() * self.pitch.cos(),
                ];
                let right = [self.yaw.cos(), 0.0, -self.yaw.sin()];
                self.target[0] += forward[0] * dy - right[0] * dx;
                self.target[1] += forward[1] * dy - right[1] * dx;
                self.target[2] += forward[2] * dy - right[2] * dx;
            }
            OrbitCameraControllerEvent::Orbit([dx, dy]) => {
                self.yaw += dx * self.yaw_speed;
                self.pitch += dy * self.pitch_speed;
                self.pitch = self.pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
            }
        }
    }
}
