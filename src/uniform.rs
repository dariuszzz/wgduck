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
}

impl UniformBindGroup {
    pub fn new(device: &wgpu::Device, stages: wgpu::ShaderStages, uniform_data: &[u8]) -> Self {
        let contents_size = std::mem::size_of_val(uniform_data) as u64;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: stages,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(contents_size),
                },
                count: None,
            }],
        });

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: uniform_data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

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
            min_size: contents_size,
        }
    }

    //this should take &mut self but the borrow checker complains in the render method lol
    pub fn update(&self, queue: &wgpu::Queue, data: &[u8]) {
        
        // crate::debug!("Uniform update");
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }
}
