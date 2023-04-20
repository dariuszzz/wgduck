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
