use wgpu::util::DeviceExt;

pub struct UniformHandle {
    pub min_size: u64,
    pub stages: wgpu::ShaderStages,
}

pub struct UniformBindGroup {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
    pub min_size: u64,
    pub max_size: u64,
}

impl UniformBindGroup {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, uniform: &Uniform) -> Self {

        let (min_size, max_size) = if let Some(DynamicInfo { min_size, max_size }) = uniform.dynamic {
            (min_size, max_size)
        } else {
            let content_size = std::mem::size_of_val(uniform.data.as_slice()) as u64;
            (content_size, content_size)
        };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: uniform.stages,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(min_size),
                },
                count: None,
            }],
        });


        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: max_size,
            label: None,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        queue.write_buffer(&buffer, 0, &uniform.data);
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {  
            bind_group_layout,
            bind_group,
            buffer,
            min_size,
            max_size,
        }
    }

    //this should take &mut self but the borrow checker complains in the render method lol
    pub fn update(&self, queue: &wgpu::Queue, data: &[u8]) {
        
        crate::debug!(format!("Uniform update {}B / {}B", data.len(), self.max_size));
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }
}

#[derive(Clone)]
pub struct DynamicInfo {
    pub min_size: u64,
    pub max_size: u64,
}

#[derive(Clone)]
pub struct Uniform {
    pub data: Vec<u8>,
    pub stages: wgpu::ShaderStages,
    // Must set for dynamic arrays (size + 1 el of unbound array)
    // None for normal ones
    pub dynamic: Option<DynamicInfo>,
}