use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use ordered_float::NotNan;
use wgpu::{
    Device, Queue, Surface, SurfaceConfiguration,
    TextureFormat, 
};

use crate::mesh::{VertexLayoutInfo, Mesh, PackedMesh};
use crate::shader::{Shader, ShaderModule};
use crate::texture::Texture;
use crate::uniform::{UniformBindGroup, Uniform, DynamicInfo};
use crate::vertex::Vertex;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub fn from_arr(arr: [f32; 4]) -> Self {
        Color {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: arr[3],
        }
    }

    pub fn clamp(&self) -> Color {
        Color {
            r: if self.r > 1.0 { self.r / 255.0 } else { self.r },
            g: if self.g > 1.0 { self.g / 255.0 } else { self.g },
            b: if self.b > 1.0 { self.b / 255.0 } else { self.b },
            a: if self.a > 1.0 { self.a / 255.0 } else { self.a },
        }
    }

    pub fn to_arr(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

pub struct RenderingContext {
    pub surface: Surface,
    pub swapchain_format: TextureFormat,
    pub queue: Queue,
    pub device: Device,
    pub config: SurfaceConfiguration,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,

    //Shader memory location as key since when including a file via static theres no reason to
    //do it multiple times also avoids an expensive hash of the entire file
    pub shader_modules: HashMap<*const str, wgpu::ShaderModule>,
    pub uniform_bindings: Vec<UniformBindGroup>,
    pub textures: Vec<Texture>,
    pub render_pipelines: HashMap<RenderPipelineInfo, wgpu::RenderPipeline>,
}

impl RenderingContext {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    fn create_depth_texture(
        device: &wgpu::Device, 
        size: (u32, u32)
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, texture_view)
    }

    pub async fn new<W>(window_size: impl Into<[u32; 2]>, window: &W) -> Self
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let window_size = window_size.into();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Indigo device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Couldn't create indigo device");

        let swapchain_format = surface
            .get_supported_formats(&adapter)
            .into_iter()
            .find(|format| {
                let desc = format.describe();

                desc.srgb
            })
            .expect("Couldn't find appropriate surface");

        let config = wgpu::SurfaceConfiguration {
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window_size[0],
            height: window_size[1],
            present_mode: wgpu::PresentMode::AutoVsync,
        };

        surface.configure(&device, &config);

        //Completely arbitrary max count copied from some website lol
        //wgpu doesnt seem to have a way to query the max amount of verts per draw call
        let max_vertex_count = 65536;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex buffer"),
            size: max_vertex_count * std::mem::size_of::<[u8; 10]>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("index buffer"),
            size: max_vertex_count * std::mem::size_of::<u16>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (depth_texture, depth_texture_view) = Self::create_depth_texture(
            &device, 
            (config.width, config.height)
        );


        Self {
            queue,
            device,
            surface,
            swapchain_format,
            // pipeline,
            config,

            vertex_buffer,
            index_buffer,

            depth_texture,
            depth_texture_view,

            shader_modules: HashMap::new(),
            uniform_bindings: Vec::new(),
            textures: Vec::new(),
            render_pipelines: HashMap::new(),
        }
    }

    pub fn update_depth_texture(&mut self, new_size: (u32, u32)) {
        let (tex, view) = Self::create_depth_texture(&self.device, new_size);
        self.depth_texture = tex;
        self.depth_texture_view = view;
    }

    pub fn render_batches(
        &mut self,
        batches: Vec<(BatchInfo, PackedMesh)>,
        distinct_uniforms: Vec<Uniform>,
        texture: Option<usize>,
        clear: Option<wgpu::Color>
    ) -> Result<(), wgpu::SurfaceError>{

        let assigned_binding_ids = self.find_or_create_uniform_bindings(&distinct_uniforms);

        //Update uniforms
        for (original_idx, binding_id) in assigned_binding_ids.iter() {
            let binding = self.uniform_bindings.get(*binding_id).unwrap();
            binding.update(&self.queue, &distinct_uniforms[*original_idx].data);
        }

        struct DrawCallData {
            pipeline_info: RenderPipelineInfo,
            textures: Vec<usize>,
            uniform_binding_ids: Vec<usize>,
            //Start and end offsets into the vertex and index buffers
            vertex_offsets: (u64, u64),
            index_offsets: (u64, u64),
            //Total amount of indices (could be computed later from offsets but w/e)
            index_count: u32,
        }

        let mut full_vert_data = Vec::new();
        let mut full_index_data = Vec::new();

        let draw_calls = batches
            .into_iter()
            //Add padding (since apparently copy buffers need to have an alignment of 4)
            //this has to be done before calculating the offset so sadly 2 maps are needed 
            .map(|(info, mut mesh)| {
                while mesh.indices.len() % 4 != 0 {
                    mesh.indices.push(*mesh.indices.last().unwrap())
                }

                (info, mesh)
            })
            .scan((0, 0), |(vert_end_offset, index_end_offset), (info, mut mesh)| {
                
                let vertex_count = mesh.vertices.len();
                let index_count = mesh.indices.len();

                //Calculate end offsets into the vertex/index buffers for each mesh
                let size_of_vert = std::mem::size_of::<u8>() * vertex_count;
                let size_of_idx = std::mem::size_of::<u16>() * index_count;
                *vert_end_offset += size_of_vert;
                *index_end_offset += size_of_idx;
                //Compute the starting offset for current mesh
                let vert_start_offset = *vert_end_offset - std::mem::size_of::<u8>() * vertex_count;
                let index_start_offset = *index_end_offset - std::mem::size_of::<u16>() * index_count;
    
                full_vert_data.append(&mut mesh.vertices);
                full_index_data.append(&mut mesh.indices);
                
                Some((
                    *vert_end_offset,
                    *index_end_offset,
                    vert_start_offset,
                    index_start_offset,
                    info,
                    index_count,
                ))
            })
            .map(|(
                vert_end_offset, 
                index_end_offset, 
                vert_start_offset,
                index_start_offset,
                batch_info, 
                index_count,
            )| {
                let pipeline_binding_ids = batch_info.distinct_uniform_ids
                    .iter()
                    // Since distinct uniforms id contains ids into the user given vector,
                    // we have to map them to the bindings assigned by find_or_create_uniform_bindings
                    .map(|id| assigned_binding_ids
                            .iter()
                            .find(|(original_idx, _)| original_idx == id)
                            .unwrap()
                            //The value is (original_idx, assigned_idx) thus .1 gets the assigned index
                            .1
                    )
                    .collect::<Vec<_>>();
                
                let pipeline_info = RenderPipelineInfo {
                    vertex_layout: batch_info.layout,
                    shader: batch_info.shader,
                    textures: batch_info.textures.clone(),
                    uniform_binding_ids: pipeline_binding_ids,
                };
                
                self.create_pipeline_if_doesnt_exist(&pipeline_info);                
                
                DrawCallData {
                    pipeline_info,
                    textures: batch_info.textures,
                    uniform_binding_ids: batch_info.distinct_uniform_ids,
                    vertex_offsets: (vert_start_offset as u64, vert_end_offset as u64),
                    index_offsets: (index_start_offset as u64, index_end_offset as u64),
                    index_count: index_count as u32,
                }
            }).collect::<Vec<_>>();

        self.queue.write_buffer(
            &self.vertex_buffer, 
            0,
            bytemuck::cast_slice(&full_vert_data)
        );

        self.queue.write_buffer(
            &self.index_buffer, 
            0,
            bytemuck::cast_slice(&full_index_data)
        );


        let mut encoder = self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("rendering encoder"),
            });

        let output = self.surface.get_current_texture()?;
        let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let output_tex = match texture {
            Some(id) => &self.textures.get(id).expect("Texture does not exist").texture_view,
            None => &output_view,
        };


        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_tex,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: match clear {
                        Some(color) => wgpu::LoadOp::Clear(color),
                        _ => wgpu::LoadOp::Load,
                    },
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: match clear {
                        Some(color) => wgpu::LoadOp::Clear(1.0),
                        _ => wgpu::LoadOp::Load,
                    },
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        
        for draw_call in draw_calls.into_iter() {

            let (v_start, v_end) = draw_call.vertex_offsets;
            let (i_start, i_end) = draw_call.index_offsets;

            if v_start == v_end || i_start == i_end { continue } 
            
            
            let pipeline = self.render_pipelines.get(&draw_call.pipeline_info).unwrap();
    
            render_pass.set_pipeline(pipeline);
    
            let mut bind_group_idx = 0;
            
            //Rework this
            for uniform_binding_id in draw_call.uniform_binding_ids.iter() {
                let assigned_binding = self.uniform_bindings.get(*uniform_binding_id).unwrap();
                render_pass.set_bind_group(bind_group_idx, &assigned_binding.bind_group, &[]);
                bind_group_idx += 1;
            }
    
            for texture_index in draw_call.textures.iter() {
                let texture = self.textures.get(*texture_index).unwrap();
                render_pass.set_bind_group(bind_group_idx, &texture.bind_group, &[]);
                bind_group_idx += 1;
            }
    
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(v_start..v_end));
            render_pass.set_index_buffer(self.index_buffer.slice(i_start..i_end), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..draw_call.index_count, 0, 0..1);
        }

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        
        match texture {
            Some(_) => {}
            None => output.present(),
        };


        Ok(())
    }

    pub fn find_or_create_uniform_bindings(
        &mut self,
        uniforms: &[Uniform],
    ) -> Vec<(usize, usize)> {
        let mut chosen_bindings: Vec<(usize, usize)> = Vec::new();

        let mut uniform_sizes = uniforms
            .iter()
            .cloned()
            .enumerate()
            .map(|(original_idx, Uniform { data, dynamic, .. })| (original_idx, (std::mem::size_of::<u8>() * data.len()) as u64, dynamic))
            .collect::<Vec<_>>();
        //Sort from highest to lowest size in order to take up bindings with big buffers first

        //this is so low size uniforms dont take the biggest buffers which would force creation
        //of unneccessary big buffers
        uniform_sizes.sort_by(|(_, a, _), (_, b, _)| b.cmp(a));

        'outer: for (uniform_idx, (original_idx, uniform_size, dynamic)) in uniform_sizes.into_iter().enumerate() {
            for binding_idx in 0..self.uniform_bindings.len() {
                //If the binding's buffer can accommodate the given uniform
                //and it hasnt been chosen already
                let binding = self.uniform_bindings.get(binding_idx).unwrap();

                //if the uniform is dynamic
                if let Some(DynamicInfo { min_size, max_size }) = dynamic {
                    // only pick dynamic buffers
                    if binding.min_size <= min_size && binding.max_size >= max_size && !chosen_bindings.contains(&(original_idx, binding_idx)) {
                        chosen_bindings.push((original_idx, binding_idx));
                        continue 'outer;
                    }
                } else {
                    // if the uniform is not dynamic then dont pick dynamic buffers
                    if binding.min_size <= uniform_size && binding.max_size >= uniform_size && !chosen_bindings.contains(&(original_idx, binding_idx)) {
                        chosen_bindings.push((original_idx, binding_idx));
                        continue 'outer;
                    }
                }
            }

            //If there was no appropriate binding available then create a new one
            let new_binding = UniformBindGroup::new(
                &self.device, 
                &self.queue,
                &uniforms[uniform_idx]
            );
            self.uniform_bindings.push(new_binding);
            //and add it to the chosen list
            chosen_bindings.push((original_idx, self.uniform_bindings.len() - 1));
            crate::debug!("Created new binding");
        }

        chosen_bindings
    }

    pub fn update_surface(&mut self, new_size: (u32, u32)) {
        self.config.width = new_size.0;
        self.config.height = new_size.1;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn create_shader_module_if_doesnt_exist(&mut self, shader_contents: &str) {
        let shader_location = shader_contents as *const _;
        
        if self.shader_modules.contains_key(&shader_location) {
            return
        }

        let module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader_contents),
                source: wgpu::ShaderSource::Wgsl(shader_contents.into()),
            });

        self.shader_modules.insert(shader_location, module);
        crate::debug!("Created new shader module");

    }

    pub fn create_pipeline_if_doesnt_exist(&mut self, pipeline_info: &RenderPipelineInfo) {
        if self.render_pipelines.contains_key(pipeline_info) {
            return;
        }

        let uniform_layouts = pipeline_info
            .uniform_binding_ids
            .iter()
            .map(|idx| &self.uniform_bindings.get(*idx).unwrap().bind_group_layout);

        let texture_layouts = pipeline_info
            .textures
            .iter()
            .filter_map(|index| self.textures.get(*index))
            .map(|tex| &tex.bind_group_layout);

        let layouts = uniform_layouts.chain(texture_layouts).collect::<Vec<_>>();

        let rp_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &layouts,
                push_constant_ranges: &[],
            });

        let (vert_module, frag_module) = match &pipeline_info.shader.modules {
            ShaderModule::Single { module } => {
                let module = self.shader_modules.get(module).expect("No shader found??");
                (module, module)
            }
            ShaderModule::Separate { vertex, fragment } => {
                let vert = self.shader_modules.get(vertex).expect("No shader found??");
                let frag = self
                    .shader_modules
                    .get(fragment)
                    .expect("No shader found??");
                (vert, frag)
            }
        };

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&rp_layout),
                vertex: wgpu::VertexState {
                    module: vert_module,
                    entry_point: &pipeline_info.shader.vert_entry,
                    buffers: &[pipeline_info.vertex_layout.descriptor()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: frag_module,
                    entry_point: &pipeline_info.shader.frag_entry,
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.swapchain_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                operation: wgpu::BlendOperation::Add,
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            },
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Self::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default()
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        self.render_pipelines.insert(pipeline_info.clone(), pipeline);
        crate::debug!("Created new pipeline");  
    }

    pub fn load_shader(
        &mut self,
        vertex_shader: &str,
        vertex_entry: &str,
        fragment_shader: &str,
        fragment_entry: &str,
    ) -> Shader {
        self.create_shader_module_if_doesnt_exist(vertex_shader);
        self.create_shader_module_if_doesnt_exist(fragment_shader);

         Shader::new(
            vertex_shader,
            vertex_entry.to_owned(),
            fragment_shader,
            fragment_entry.to_owned(),
        )
    }

    pub fn create_texture(&mut self, data: &[u8], dimensions: (u32, u32)) -> usize {
        let texture = Texture::new(
            &self.device,
            &self.queue,
            data,
            dimensions
        );

        self.textures.push(texture);
        let index = self.textures.len() - 1;

        index
    }

    pub fn update_texture(
        &mut self, 
        texture_handle: usize, 
        data: &[u8], 
    ) {
        if let Some(texture) = self.textures.get_mut(texture_handle) { 
            texture.update(&self.queue, data);
        }
    }
}

#[derive(Eq, Clone)]
pub struct RenderPipelineInfo {
    vertex_layout: VertexLayoutInfo,
    shader: Shader,
    textures: Vec<usize>,
    uniform_binding_ids: Vec<usize>,
}

impl PartialEq for RenderPipelineInfo {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_layout == other.vertex_layout
        && self.shader == other.shader
        && self.textures.len() == other.textures.len()
        && self.uniform_binding_ids == other.uniform_binding_ids
    }
}

impl Hash for RenderPipelineInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vertex_layout.hash(state);
        self.shader.hash(state);
        self.textures.len().hash(state);
        self.shader.hash(state);
        self.uniform_binding_ids.hash(state);
    }
}

#[derive(Eq, Clone)]
pub struct BatchInfo {
    pub layout: VertexLayoutInfo,
    pub shader: Shader,
    pub textures: Vec<usize>,
    pub distinct_uniform_ids: Vec<usize>,
    pub transparent: bool,
}

impl PartialEq for BatchInfo {
    fn eq(&self, other: &Self) -> bool {
        self.layout == other.layout
        && self.shader == other.shader
        && self.textures.len() == other.textures.len()
        && self.distinct_uniform_ids == other.distinct_uniform_ids
        && self.transparent == other.transparent
    }
}

impl Hash for BatchInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.layout.hash(state);
        self.shader.hash(state);
        self.textures.len().hash(state);
        self.distinct_uniform_ids.hash(state);
        self.transparent.hash(state);
    }
}

impl BatchInfo {
    pub fn new(
        mesh: &PackedMesh,
        shader: Shader,
        textures: Vec<usize>,
        distinct_uniform_ids: Vec<usize>
    ) -> Self {
        Self {
            layout: mesh.layout.clone(),
            shader,
            textures,
            distinct_uniform_ids,
            transparent: mesh.could_be_transparent,
        }
    }
}