use std::marker::PhantomData;

use glm::Vec3;
use itertools::Itertools;
use wgpu::VertexFormat;


use crate::vertex::{Vertex};

use nalgebra_glm as glm;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VertexLayoutInfo {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
    pub total_size: wgpu::BufferAddress,
}

impl VertexLayoutInfo {
    pub fn descriptor(&self) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: self.step_mode,
            attributes: &self.attributes,
        }
    }

    pub fn from_vertex<V: Vertex>() -> Self {
        let mut total_size = 0 as wgpu::BufferAddress;
        let attributes = V::fields()
                .into_iter()
                .scan(0, |offset, vformat| {
                    total_size += vformat.size();
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
            array_stride: total_size,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes,
            total_size,
        }
    }
}

#[derive(Clone)]
pub struct Mesh<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u16>,
    pub layout: VertexLayoutInfo,
    pub could_be_transparent: bool,
}


impl Mesh<u8> {
    pub fn merge(&mut self, other: &mut Mesh<u8>) {
        assert!(self.layout == other.layout);

        other.indices.iter_mut().for_each(|i| {
            *i += self.vertices.len() as u16 / self.layout.total_size as u16;
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


        if other.could_be_transparent {
            self.could_be_transparent = true;
        }
    }
}

impl<V: Vertex> Mesh<V> {
    pub fn merge(&mut self, other: &mut Mesh<V>) {
        assert!(self.layout == other.layout);

        other.indices.iter_mut().for_each(|i| {
            *i += self.vertices.len() as u16;
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


        if other.could_be_transparent {
            self.could_be_transparent = true;
        }
    }
}

impl<V: Vertex> Mesh<V> {


    pub fn new(vertices: Vec<V>, indices: Vec<u16>, could_be_transparent: bool) -> Self {
        
        Self {
            vertices: bytemuck::cast_slice(&vertices).to_vec(),
            indices,
            layout: VertexLayoutInfo::from_vertex::<V>(),
            could_be_transparent,
        }
    }

    pub fn apply_rotation(&self, rotation: glm::Quat) -> Self {
        let mut vertices = self.vertices.clone();
        for vert in &mut vertices {
            let pos = vert.position();
            let rotated_pos = glm::quat_rotate_vec3(&rotation, &pos);
            vert.set_position(rotated_pos);
        }
        
        Self {
            vertices,
            ..self.clone()
        }
    }

    pub fn apply_translation(&self, translation: glm::Vec3) -> Self {
        let mut vertices = self.vertices.clone(); 
        for vert in &mut vertices {
            let pos = vert.position();
            let rotated_pos = pos + translation;
            vert.set_position(rotated_pos);
        }

        Self {
            vertices,
            ..self.clone()
        }
    }

    pub fn pack(&self) -> PackedMesh {
        PackedMesh {
            vertices: bytemuck::cast_slice(&self.vertices).to_vec(),
            indices: self.indices.clone(),
            could_be_transparent: self.could_be_transparent,
            layout: self.layout.clone(),
        }
    }
}

pub type PackedMesh = Mesh<u8>;