use crate::mesh::VertexLayoutInfo;


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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BasicVertexData {
    pub pos: [f32; 3],
}

impl VertexTrait for BasicVertexData {
    fn vertex_layout() -> Vec<VertexType> {
        vec![
            VertexType::Float32x3
        ]
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Vertex<AdditionalData: VertexTrait> {
    pub base: BasicVertexData,
    pub additional: AdditionalData
}

impl<AdditionalData: VertexTrait> Vertex<AdditionalData> {
    pub fn full_layout() -> Vec<VertexType> {
        let mut base_layout = BasicVertexData::vertex_layout();
        let mut additional_layout = AdditionalData::vertex_layout();
        base_layout.append(&mut additional_layout);
        base_layout
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut base_packed: Vec<u8> = bytemuck::cast_slice(&[self.base]).to_vec();
        let mut additional_packed = bytemuck::cast_slice(&[self.additional]).to_vec();
        base_packed.append(&mut additional_packed);
        base_packed
    }
    
    pub fn new(pos: [f32; 3], additional: AdditionalData) -> Self {
        Self {
            base: BasicVertexData { pos },
            additional
        }
    }
}

