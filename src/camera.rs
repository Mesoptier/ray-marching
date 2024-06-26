use nalgebra::{convert, Affine3, Point3, Translation3, UnitQuaternion, Vector3};

pub(crate) struct Camera {
    position: Point3<f32>,
    rotation: UnitQuaternion<f32>,
}

impl Camera {
    /// Get world-to-view transformation matrix.
    pub(crate) fn view(&self) -> Affine3<f32> {
        convert(self.rotation.inverse() * Translation3::from(-self.position.coords))
    }
}

pub(crate) enum OrbitCameraControllerEvent {
    Pan([f32; 2]),
    Orbit([f32; 2]),
    Dolly(f32),
}

pub(crate) struct OrbitCameraController {
    /// Target to look at.
    target: Point3<f32>,
    /// Pitch angle in radians.
    pitch: f32,
    /// Yaw angle in radians.
    yaw: f32,
    /// Distance from the target.
    radius: f32,

    pan_speed: f32,
    yaw_speed: f32,
    pitch_speed: f32,
    dolly_speed: f32,
}

impl OrbitCameraController {
    pub(crate) fn new(target: [f32; 3], radius: f32) -> Self {
        Self {
            target: Point3::from(target),
            pitch: 0.0,
            yaw: 0.0,
            radius,

            pan_speed: 0.01,
            yaw_speed: 0.01,
            pitch_speed: 0.01,
            dolly_speed: 0.01,
        }
    }

    fn rotation(&self) -> UnitQuaternion<f32> {
        UnitQuaternion::from_euler_angles(-self.pitch, -self.yaw, 0.0)
    }

    pub(crate) fn camera(&self) -> Camera {
        let rotation = self.rotation();
        let position = self.target + rotation * Vector3::z() * self.radius;
        Camera { position, rotation }
    }

    pub(crate) fn update(&mut self, event: OrbitCameraControllerEvent) {
        match event {
            OrbitCameraControllerEvent::Pan([dx, dy]) => {
                // Right and up vectors relative to camera orientation.
                let right = self.rotation() * Vector3::x();
                let up = self.rotation() * Vector3::y();
                // TODO: Make pan speed proportional to distance from target?
                self.target += (right * -dx + up * dy) * self.pan_speed;
            }
            OrbitCameraControllerEvent::Orbit([dx, dy]) => {
                self.yaw += dx * self.yaw_speed;
                self.pitch += dy * self.pitch_speed;

                // Clamping to value slightly less than pi/2 to avoid gimbal lock.
                // TODO: Fix shader to support quaternion rotation.
                self.pitch = self.pitch.clamp(-1.5, 1.5);
            }
            OrbitCameraControllerEvent::Dolly(delta) => {
                self.radius += delta * self.dolly_speed * self.radius;
                self.radius = self.radius.max(0.1);
            }
        }
    }
}
