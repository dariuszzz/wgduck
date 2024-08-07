use encase::ShaderType;
use nalgebra_glm as glm;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: glm::Mat4::identity().into(),
        }
    }
}

pub struct OrthoCamera {
    pub pos: glm::Vec3,
    pub target: glm::Vec3,
    pub up: glm::Vec3,
    pub znear: f32,
    pub zfar: f32,
    pub zoom: f32,

    pub uniform: CameraUniform,
}

impl OrthoCamera {
    pub fn update(&mut self, new_window_size: &glm::UVec2) {
        self.uniform.view_proj = self.build_view_proj_matrix(new_window_size).into();
    }

    pub fn build_view_matrix(&self) -> glm::Mat4 {
        glm::look_at_rh(&self.pos, &self.target, &self.up)
    }

    pub fn build_proj_matrix(&self, new_window_size: &glm::UVec2) -> glm::Mat4 {
        let width = new_window_size.x as f32;
        let height = new_window_size.y as f32;

        let scale_x = width / self.zoom;
        let scale_y = height / self.zoom;

        let proj = glm::ortho(-scale_x, scale_x, -scale_y, scale_y, -self.zfar, self.zfar);

        proj
    }

    pub fn build_view_proj_matrix(&self, window_size: &glm::UVec2) -> glm::Mat4 {
        let view = self.build_view_matrix();
        let proj = self.build_proj_matrix(window_size);

        proj * view
    }

    pub fn new(
        pos: glm::Vec3,
        target: glm::Vec3,
        up: glm::Vec3,
        znear: f32,
        zfar: f32,
        zoom: f32,
    ) -> Self {
        let uniform = CameraUniform::default();

        Self {
            pos,
            target,
            up,
            zfar,
            znear,
            uniform,
            zoom,
        }
    }
}

pub struct PerspectiveCamera {
    pub pos: glm::Vec3,
    pub target: glm::Vec3,
    pub up: glm::Vec3,
    pub near: f32,
    pub far: f32,
    pub fov: f32,

    pub uniform: CameraUniform,
}

impl PerspectiveCamera {
    pub fn update(&mut self, window_size: &glm::UVec2) {
        self.uniform.view_proj = self.build_view_proj_matrix(window_size).into();
    }

    pub fn build_view_matrix(&self) -> glm::Mat4 {
        glm::look_at_rh(&self.pos, &self.target, &self.up)
    }

    pub fn build_proj_matrix(&self, window_size: &glm::UVec2) -> glm::Mat4 {
        let width = window_size.x as f32;
        let height = window_size.y as f32;

        let aspect_ratio = width / height;

        glm::perspective(aspect_ratio, self.fov, self.near, self.far)
    }

    pub fn build_view_proj_matrix(&self, window_size: &glm::UVec2) -> glm::Mat4 {
        let view = self.build_view_matrix();
        let proj = self.build_proj_matrix(window_size);

        proj * view
    }

    pub fn new(
        pos: glm::Vec3,
        target: glm::Vec3,
        up: glm::Vec3,
        near: f32,
        far: f32,
        fov: f32,
    ) -> Self {
        let uniform = CameraUniform::default();

        Self {
            pos,
            target,
            up,
            far,
            near,
            uniform,
            fov,
        }
    }
}
