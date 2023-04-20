#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

pub struct Camera {
    pub pos: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub znear: f32,
    pub zfar: f32,

    pub uniform: CameraUniform,
}

impl Camera {
    pub fn update(&mut self, new_window_size: (u32, u32)) {
        self.uniform.view_proj = self.build_view_proj_matrix(new_window_size).into();
    }

    pub fn build_view_proj_matrix(&self, new_window_size: (u32, u32)) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.pos, self.target, self.up);

        let width = new_window_size.0 as f32;
        let height = new_window_size.1 as f32;

        // let aspect_ratio = width / height;

        let proj = cgmath::ortho(0.0, width, height, 0.0, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn new(
        pos: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let uniform = CameraUniform::default();

        Self {
            pos,
            target,
            up,
            zfar,
            znear,
            uniform,
        }
    }
}
