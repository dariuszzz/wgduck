use encase::{internal::WriteInto, ShaderSize, ShaderType};
use nalgebra_glm as glm;
use wgpu::VertexFormat;

pub trait Vertex: bytemuck::Pod + bytemuck::Zeroable {
    fn fields() -> Vec<VertexFormat>;
    fn position(&self) -> glm::Vec3;
    fn set_position(&mut self, pos: glm::Vec3);
}
