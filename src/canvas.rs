use ggez::glam::*;
use ggez::graphics::{Color, Shader};
use ggez::{graphics, Context};
use wgpu::util::DeviceExt;

use crate::camera::CameraBundle;
use crate::mesh::Vertex;
use crate::{camera::CameraUniform, prelude::*};

#[derive(Clone)]
pub struct DrawState3d {
    pub position: Vec3,
    pub shader: Shader,
}

#[derive(Clone)]
pub struct DrawCommand3d {
    pub mesh: Mesh3d,
    pub state: DrawState3d,
}

pub struct Canvas3d {
    pub draws: Vec<DrawCommand3d>,
    pub dirty_pipeline: bool,
    pub state: DrawState3d,
    pub original_state: DrawState3d,
    pub pipeline: wgpu::RenderPipeline,
    pub depth: graphics::ScreenImage,
    pub camera_bundle: CameraBundle, // TODO: Support multiple cameras by rendering to a texture. Maybe just rerender and change the camera uniform?
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
}

impl Canvas3d {
    pub fn new(ctx: &mut Context) -> Self {
        let shader = graphics::ShaderBuilder::from_path("/cube.wgsl")
            .build(&ctx.gfx)
            .unwrap();

        let mut camera_bundle = CameraBundle::default();
        camera_bundle.projection.aspect = ctx.gfx.size().0 / ctx.gfx.size().1;
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera_bundle);

        let camera_buffer =
            ctx.gfx
                .wgpu()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let camera_bind_group_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });
        let texture_bind_group_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });
        let camera_bind_group =
            ctx.gfx
                .wgpu()
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &camera_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }],
                    label: Some("camera_bind_group"),
                });

        let render_pipeline_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let depth = graphics::ScreenImage::new(ctx, graphics::ImageFormat::Depth32Float, 1., 1., 1);

        let pipeline3d = Canvas3d {
            depth,
            dirty_pipeline: false,
            camera_bundle,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            state: DrawState3d {
                shader: shader.clone(),
                position: Vec3::ZERO,
            },
            original_state: DrawState3d {
                shader: shader.clone(),
                position: Vec3::ZERO,
            },
            draws: Vec::default(),
            pipeline: ctx.gfx.wgpu().device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader.vs_module.unwrap(),
                        entry_point: "vs_main",
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vertex>() as _,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                // pos
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                // tex_coord
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x2,
                                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                    shader_location: 1,
                                },
                            ],
                        }],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader.fs_module.unwrap(),
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: ctx.gfx.surface_format(),
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                },
            ),
        };

        pipeline3d
    }

    pub fn set_default_shader(&mut self, ctx: &mut Context) {
        let shader = graphics::ShaderBuilder::from_path("/cube.wgsl")
            .build(&ctx.gfx)
            .unwrap();
        // let params = graphics::ShaderParamsBuilder::new(&false).build(ctx);
        self.state.shader = shader;
        self.dirty_pipeline = true;
    }

    pub fn set_shader(&mut self, shader: Shader) {
        self.state.shader = shader;
        self.dirty_pipeline;
    }

    pub fn update_pipeline(&mut self, ctx: &mut Context) {
        let camera_bind_group_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });
        let texture_bind_group_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });
        let render_pipeline_layout =
            ctx.gfx
                .wgpu()
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        self.pipeline =
            ctx.gfx
                .wgpu()
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &self
                            .state
                            .shader
                            .vs_module
                            .as_ref()
                            .unwrap_or(self.original_state.shader.vs_module.as_ref().unwrap()),
                        entry_point: "vs_main",
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vertex>() as _,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                // pos
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                // tex_coord
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x2,
                                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                    shader_location: 1,
                                },
                            ],
                        }],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &self
                            .state
                            .shader
                            .fs_module
                            .as_ref()
                            .unwrap_or(self.original_state.shader.fs_module.as_ref().unwrap()),
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: ctx.gfx.surface_format(),
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                });
    }

    pub fn finish(&mut self, ctx: &mut Context, clear_color: Color) {
        if self.dirty_pipeline {
            self.update_pipeline(ctx);
        }

        {
            let depth = self.depth.image(ctx).clone();
            let frame = ctx.gfx.frame().clone();
            let mut pass =
                ctx.gfx
                    .commands()
                    .unwrap()
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: frame.wgpu().1,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    graphics::LinearColor::from(clear_color).into(),
                                ),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: depth.wgpu().1,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });
            for draw in self.draws.iter() {
                if draw.state.shader != self.state.shader {
                    // self.set_shader(draw.state.shader.clone());
                }
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, draw.mesh.bind_group.as_ref().unwrap(), &[]);
                pass.set_bind_group(1, &self.camera_bind_group, &[]);
                pass.set_vertex_buffer(0, draw.mesh.vert_buffer.as_ref().unwrap().slice(..));
                pass.set_index_buffer(
                    draw.mesh.ind_buffer.as_ref().unwrap().slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                pass.draw_indexed(0..draw.mesh.indices.len() as u32, 0, 0..1);
            }
        }
        self.draws.clear();
    }

    pub fn draw(&mut self, mesh: Mesh3d) {
        self.draws.push(DrawCommand3d {
            mesh,
            state: self.state.clone(),
        });
    }

    pub fn resize(&mut self, width: f32, height: f32, ctx: &mut Context) {
        self.camera_bundle
            .projection
            .resize(width as u32, height as u32);
        self.camera_uniform.update_view_proj(&self.camera_bundle);
        ctx.gfx.wgpu().queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn update_camera(&mut self, ctx: &mut Context) {
        self.camera_uniform.update_view_proj(&self.camera_bundle);
        ctx.gfx.wgpu().queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}