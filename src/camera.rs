use std::f32::consts::PI;

use cgmath::{Deg, Rad};

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

pub struct OrthoCamera {
    pub pos: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub znear: f32,
    pub zfar: f32,
    pub zoom: f32,

    pub uniform: CameraUniform,
}

impl OrthoCamera {
    pub fn update(&mut self, new_window_size: (u32, u32)) {
        self.uniform.view_proj = self.build_view_proj_matrix(new_window_size).into();
    }

    pub fn build_view_proj_matrix(&self, new_window_size: (u32, u32)) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.pos, self.target, self.up);

        let width = new_window_size.0 as f32;
        let height = new_window_size.1 as f32;

        let scale_x = width / self.zoom;
        let scale_y = height / self.zoom;


        let proj = cgmath::ortho(-scale_x, scale_x, -scale_y, scale_y, -self.zfar, self.zfar);
        // let proj = cgmath::ortho(0.0, width/self.zoom, 0.0, height/self.zoom, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn new(
        pos: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
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
    pub pos: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub near: f32,
    pub far: f32,
    pub fov: f32,

    pub uniform: CameraUniform,
}

impl PerspectiveCamera {
    pub fn update(&mut self, new_window_size: (u32, u32)) {
        self.uniform.view_proj = self.build_view_proj_matrix(new_window_size).into();
    }

    pub fn build_view_proj_matrix(&self, new_window_size: (u32, u32)) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.pos, self.target, self.up);

        let width = new_window_size.0 as f32;
        let height = new_window_size.1 as f32;

        let aspect_ratio = width / height;
        
        let diag = ((height*height)+(width*width)).sqrt();
        let fov = 2.0 * ((diag) / (2.0 * self.far)).atan();
        let proj = cgmath::perspective(Rad(fov), aspect_ratio, self.near, self.far);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn new(
        pos: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        near: f32,
        far: f32,
        fov: f32
    ) -> Self {
        let uniform = CameraUniform::default();

        Self {
            pos,
            target,
            up,
            far,
            near,
            uniform,
            fov
        }
    }
}
