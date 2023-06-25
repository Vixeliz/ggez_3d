pub(crate) use ggez::glam::*;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: Vec3::Y,
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

impl Camera {
    pub fn build_view_projection_matrix(&self, view: Mat4) -> Mat4 {
        // 2.
        let proj = Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        // 3.
        return proj * view;
        // OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pub view_proj: [[f32; 4]; 4],
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

    pub fn update_view_proj(&mut self, camera: &Camera, view: Mat4) {
        let view = camera.build_view_projection_matrix(view);
        self.view_proj = [
            view.x_axis.into(),
            view.y_axis.into(),
            view.z_axis.into(),
            view.w_axis.into(),
        ];
    }
}
