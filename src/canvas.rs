use ggez::graphics::{Color, Shader};
use ggez::{glam::*, GameError, GameResult};
use ggez::{graphics, Context};
use wgpu::util::DeviceExt;

use crate::camera::CameraBundle;
use crate::mesh::{Aabb, Instance3d, Transform3d, Vertex};
use crate::{camera::CameraUniform, prelude::*};

#[derive(Clone)]
pub struct DrawParam3d {
    pub transform: Transform3d,
    /// The alpha component is used for intensity of blending instead of actual alpha
    pub color: Color,
}

impl DrawParam3d {
    pub fn scale<V>(mut self, scale_: V) -> Self
    where
        V: Into<mint::Vector3<f32>>,
    {
        let p: mint::Vector3<f32> = scale_.into();
        self.transform.scale = p;
        self
    }

    pub fn position<P>(mut self, position_: P) -> Self
    where
        P: Into<mint::Vector3<f32>>,
    {
        let p: mint::Vector3<f32> = position_.into();
        self.transform.position = p;
        self
    }

    pub fn rotation<R>(mut self, rotation_: R) -> Self
    where
        R: Into<mint::Quaternion<f32>>,
    {
        let p: mint::Quaternion<f32> = rotation_.into();
        self.transform.rotation = p;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn transform(mut self, transform: Transform3d) -> Self {
        self.transform = transform;
        self
    }
}

impl Default for DrawParam3d {
    fn default() -> Self {
        Self {
            transform: Transform3d::default(),
            color: Color::new(1.0, 1.0, 1.0, 0.0),
        }
    }
}

#[derive(Clone)]
pub struct DrawState3d {
    pub shader: Shader,
}

#[derive(Clone)]
pub struct DrawCommand3d {
    pub mesh: Mesh3d, // Maybe take a reference instead
    pub state: DrawState3d,
    pub param: DrawParam3d,
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
    pub instance_buffer: wgpu::Buffer,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
}

impl Canvas3d {
    pub fn new(ctx: &mut Context) -> Self {
        let cube_code = include_str!("../resources/cube.wgsl");
        let shader = graphics::ShaderBuilder::from_code(cube_code)
            .build(&ctx.gfx)
            .unwrap(); // Should never fail since cube.wgsl is unchanging

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

        let instance_buffer =
            ctx.gfx
                .wgpu()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&[Instance3d::default(), Instance3d::default()]),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
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
            },
            original_state: DrawState3d {
                shader: shader.clone(),
            },
            draws: Vec::default(),
            pipeline: ctx.gfx.wgpu().device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader.vs_module.unwrap(), // Should never fail since it's already built
                        entry_point: "vs_main",
                        buffers: &[Vertex::desc(), Instance3d::desc()],
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
                        module: &shader.fs_module.unwrap(), // Should never fail since already built
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
            instance_buffer,
        };

        pipeline3d
    }

    pub fn set_default_shader(&mut self, ctx: &mut Context) {
        let cube_code = include_str!("../resources/cube.wgsl");
        let shader = graphics::ShaderBuilder::from_path(cube_code)
            .build(&ctx.gfx)
            .unwrap(); // Should never fail since cube.wgsl is unchanging
        self.state.shader = shader;
        self.dirty_pipeline = true;
    }

    pub fn set_shader(&mut self, shader: Shader) {
        self.state.shader = shader;
        self.dirty_pipeline = true;
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
                        module: &self.state.shader.vs_module.as_ref().unwrap_or(
                            self.original_state
                                .shader
                                .vs_module
                                .as_ref()
                                .unwrap_or(self.original_state.shader.vs_module.as_ref().unwrap()), // Should always exist
                        ),
                        entry_point: "vs_main",
                        buffers: &[
                            wgpu::VertexBufferLayout {
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
                                        offset: std::mem::size_of::<[f32; 3]>()
                                            as wgpu::BufferAddress,
                                        shader_location: 1,
                                    },
                                ],
                            },
                            Instance3d::desc(),
                        ],
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
                            .unwrap_or(self.original_state.shader.fs_module.as_ref().unwrap()), // Should always exist since we use original
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

    pub fn finish(&mut self, ctx: &mut Context, clear_color: Color) -> GameResult {
        if self.dirty_pipeline {
            self.update_pipeline(ctx);
        }

        {
            let depth = self.depth.image(ctx).clone();
            let frame = ctx.gfx.frame().clone();
            self.update_instance_data(ctx);
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
            for (i, draw) in self.draws.iter().enumerate() {
                let i = i as u32;
                if draw.state.shader != self.state.shader {
                    // self.set_shader(draw.state.shader.clone());
                    // self.update_pipeline(ctx);
                }

                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                pass.set_bind_group(
                    0,
                    draw.mesh.bind_group.as_ref().ok_or(GameError::CustomError(
                        "Bind Group not generated for mesh".to_string(),
                    ))?,
                    &[],
                );
                pass.set_bind_group(1, &self.camera_bind_group, &[]);
                pass.set_vertex_buffer(
                    0,
                    draw.mesh
                        .vert_buffer
                        .as_ref()
                        .ok_or(GameError::CustomError(
                            "Vert Buffer not generated for mesh".to_string(),
                        ))?
                        .slice(..),
                );
                pass.set_index_buffer(
                    draw.mesh
                        .ind_buffer
                        .as_ref()
                        .ok_or(GameError::CustomError(
                            "Ind Buffer not generated for mesh".to_string(),
                        ))?
                        .slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                pass.draw_indexed(0..draw.mesh.indices.len() as u32, 0, i..i + 1);
            }
        }
        self.draws.clear();
        Ok(())
    }

    pub fn update_instance_data(&mut self, ctx: &mut Context) {
        let instance_data = self
            .draws
            .iter()
            .map(|x| {
                Instance3d::from_param(&x.param, x.mesh.to_aabb().unwrap_or(Aabb::default()).center)
            })
            .collect::<Vec<_>>();
        ctx.gfx.wgpu().queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );
    }

    pub fn draw(&mut self, mesh: Mesh3d, param: DrawParam3d) {
        self.draws.push(DrawCommand3d {
            mesh,
            state: self.state.clone(),
            param,
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
