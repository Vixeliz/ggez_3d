pub(crate) use ggez::glam::*;

#[derive(Default)]
pub struct CameraBundle {
    pub camera: Camera,
    pub projection: Projection,
}

#[derive(Debug, Default)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new<V: Into<Vec3>>(position: V, yaw: f32, pitch: f32) -> Self {
        Self {
            position: position.into(),
            yaw,
            pitch,
        }
    }

    pub fn calc_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        look_to_rh(
            self.position,
            Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vec3::Y,
        )
    }
}

pub fn look_to_rh(eye: Vec3, dir: Vec3, up: Vec3) -> Mat4 {
    let f = dir.normalize();
    let s = f.cross(up).normalize();
    let u = s.cross(f);

    #[rustfmt::skip]
      let mat =   Mat4::from_cols_array(&[
            s.x, u.x, -f.x, 0.0,
            s.y, u.y, -f.y, 0.0,
            s.z, u.z, -f.z, 0.0,
            -eye.dot(s), -eye.dot(u), eye.dot(f), 1.0,
        ]
    );

    mat
}

pub struct Projection {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Default for Projection {
    fn default() -> Self {
        Self::new(1920, 1080, 70.0_f32.to_radians(), 0.1, 100.0)
    }
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: [
                Mat4::IDENTITY.x_axis.into(),
                Mat4::IDENTITY.y_axis.into(),
                Mat4::IDENTITY.z_axis.into(),
                Mat4::IDENTITY.w_axis.into(),
            ],
        }
    }

    pub fn update_view_proj(&mut self, camera_bundle: &CameraBundle) {
        let view = camera_bundle.projection.calc_matrix() * camera_bundle.camera.calc_matrix();
        self.view_proj = [
            view.x_axis.into(),
            view.y_axis.into(),
            view.z_axis.into(),
            view.w_axis.into(),
        ];
    }
}
