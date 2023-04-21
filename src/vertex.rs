// #[repr(C)]
// #[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq, Default)]
// pub struct Vertex {
//     pub pos: [f32; 3],
//     pub tint_color: [f32; 4],
// }

// impl Vertex {
//     const ATTRIBS: [wgpu::VertexAttribute; 2] =
//         wgpu::vertex_attr_array![ 0 => Float32x3, 1 => Float32x4 ];

//     pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         wgpu::VertexBufferLayout {
//             array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &Self::ATTRIBS,
//         }
//     }
// }

pub enum VertexType {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,
}

pub trait VertexTrait: bytemuck::Pod + bytemuck::Zeroable {
    fn vertex_layout() -> Vec<VertexType>;
}