use wgpu::VertexFormat;

use crate::vertex::{VertexTrait, VertexType};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VertexLayoutInfo {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}

impl VertexLayoutInfo {
    pub fn descriptor(&self) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: self.step_mode,
            attributes: &self.attributes,
        }
    }

    pub fn from_vertex<V: VertexTrait>() -> Self {
        let attributes = V::vertex_layout()
                .into_iter()
                .map(|vtype| match vtype {
                    VertexType::Float32 => VertexFormat::Float32,
                    VertexType::Float32x2 => VertexFormat::Float32x2,
                    VertexType::Float32x3 => VertexFormat::Float32x3,
                    VertexType::Float32x4 => VertexFormat::Float32x4,
                    VertexType::Uint32 => VertexFormat::Uint32,
                    VertexType::Uint32x2 => VertexFormat::Uint32x2,
                    VertexType::Uint32x3 => VertexFormat::Uint32x3,
                    VertexType::Uint32x4 => VertexFormat::Uint32x4,
                    VertexType::Sint32 => VertexFormat::Sint32,
                    VertexType::Sint32x2 => VertexFormat::Sint32x2,
                    VertexType::Sint32x3 => VertexFormat::Sint32x3,
                    VertexType::Sint32x4 => VertexFormat::Sint32x4,
                })
                .scan(0, |offset, vformat| {
                    *offset += vformat.size();
                    Some((*offset, vformat))
                })
                .enumerate()
                .map(|(index, (offset, vformat))| {
                    wgpu::VertexAttribute {
                        // Need to subtract the current format size in order to for the offsets
                        // to start at zero
                        offset: offset - vformat.size(),
                        shader_location: index as u32,
                        format: vformat,
                    }
                })
                .collect::<Vec<_>>();

        Self {
            array_stride: std::mem::size_of::<V>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes,
        }
    }
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<u8>,
    pub indices: Vec<u16>,
    pub layout: VertexLayoutInfo,
    pub could_be_transparent: bool,
    pub highest_z: f32,
}

impl Mesh {
    pub fn merge(&mut self, other: &mut Mesh) {
        assert_eq!(
            self.layout, other.layout,
            "cant merge two meshes of different vertex types"
        );

        other.indices.iter_mut().for_each(|i| {
            //Account for the fact that we are working on vertex data cast to [u8] by
            //dividing the amount of vertices by the stride
            *i += (self.vertices.len() / self.layout.array_stride as usize) as u16;
        });

        // deconstructed triangle between meshes
        if let Some(last_index_from_other) = other.indices.last().cloned() {
            if let Some(last_self_index) = self.indices.last().cloned() {
                self.indices.append(&mut vec![
                    last_self_index,
                    last_self_index,
                    last_index_from_other,
                ]);
            }
        }

        self.indices.append(&mut other.indices);
        self.vertices.append(&mut other.vertices);

        if other.highest_z > self.highest_z { 
            self.highest_z = other.highest_z;
        }

        if other.could_be_transparent {
            self.could_be_transparent = true;
        }
    }
}
