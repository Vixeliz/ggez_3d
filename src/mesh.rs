use ggez::graphics::Image;
use ggez::{graphics, Context};
use glam::{Mat4, Vec3};
use mint::{Vector2, Vector3};
use std::sync::Arc;
use wgpu::util::DeviceExt;

use crate::canvas::DrawParam3d;

#[derive(Debug, Copy, Clone)]
pub struct Aabb {
    pub center: mint::Vector3<f32>,
    pub half_extents: mint::Vector3<f32>,
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO.into(),
            half_extents: Vec3::ZERO.into(),
        }
    }
}

impl Aabb {
    #[inline]
    pub fn from_min_max(minimum: Vec3, maximum: Vec3) -> Self {
        let minimum = Vec3::from(minimum);
        let maximum = Vec3::from(maximum);
        let center = 0.5 * (maximum + minimum);
        let half_extents = 0.5 * (maximum - minimum);
        Self {
            center: center.into(),
            half_extents: half_extents.into(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Transform3d {
    pub position: mint::Vector3<f32>,
    pub rotation: mint::Quaternion<f32>,
    pub scale: mint::Vector3<f32>,
}

impl Default for Transform3d {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0).into(),
            rotation: glam::Quat::IDENTITY.into(),
            scale: Vec3::new(1.0, 1.0, 1.0).into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance3d {
    transform: [[f32; 4]; 4],
    color: [f32; 4],
}

impl Default for Instance3d {
    fn default() -> Self {
        Self::from_param(&DrawParam3d::default(), Vec3::ZERO)
    }
}

impl Instance3d {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance3d>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
    pub fn from_param<V>(param: &DrawParam3d, center: V) -> Self
    where
        V: Into<mint::Vector3<f32>>,
    {
        let pivot: mint::Vector3<f32> = center.into();
        let transform =
            Mat4::from_translation(Vec3::from(param.transform.position) + Vec3::from(pivot))
                * Mat4::from_scale(param.transform.scale.into())
                * Mat4::from_quat(param.transform.rotation.into())
                * Mat4::from_translation(
                    (Vec3::from(param.transform.position) + Vec3::from(pivot)) * -1.0,
                );

        Self {
            transform: [
                transform.x_axis.into(),
                transform.y_axis.into(),
                transform.z_axis.into(),
                transform.w_axis.into(),
            ],
            color: param.color.into(),
        }
    }
}

// TODO: Allow custom vertex formats
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub tex_coord: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn new<V, T, C>(position: V, uv: T, color: C) -> Vertex
    where
        V: Into<Vector3<f32>>,
        T: Into<Vector2<f32>>,
        C: Into<Option<graphics::Color>>,
    {
        let position: Vector3<f32> = position.into();
        let uv: Vector2<f32> = uv.into();
        let color: Option<graphics::Color> = color.into();
        let color = color
            .unwrap_or(graphics::Color::new(1.0, 1.0, 1.0, 0.0))
            .into();
        Vertex {
            pos: position.into(),
            tex_coord: uv.into(),
            color,
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
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
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                //color
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
            ],
        }
    }
}

#[derive(Clone)]
pub struct Mesh3d {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vert_buffer: Option<Arc<wgpu::Buffer>>,
    pub ind_buffer: Option<Arc<wgpu::Buffer>>,
    pub bind_group: Option<Arc<wgpu::BindGroup>>,
    pub texture: Option<Image>,
}

impl Mesh3d {
    pub fn gen_wgpu_buffer(&mut self, pipeline: &wgpu::RenderPipeline, ctx: &mut Context) {
        let verts = ctx
            .gfx
            .wgpu()
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(self.vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let inds = ctx
            .gfx
            .wgpu()
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(self.indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            });

        // Allow custom one set through mesh
        let sampler = ctx
            .gfx
            .wgpu()
            .device
            .create_sampler(&graphics::Sampler::default().into());

        let bind_group = ctx
            .gfx
            .wgpu()
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            self.texture
                                .as_ref()
                                .unwrap_or(&Image::from_color(
                                    ctx,
                                    1,
                                    1,
                                    Some(graphics::Color::WHITE),
                                ))
                                .wgpu()
                                .1,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        self.bind_group = Some(Arc::new(bind_group));
        self.vert_buffer = Some(Arc::new(verts));
        self.ind_buffer = Some(Arc::new(inds));
    }

    pub fn to_aabb(&self) -> Option<Aabb> {
        let mut minimum = Vec3::MAX;
        let mut maximum = Vec3::MIN;
        for p in self.vertices.iter() {
            minimum = minimum.min(Vec3::from_array(p.pos));
            maximum = maximum.max(Vec3::from_array(p.pos));
        }
        if minimum.x != std::f32::MAX
            && minimum.y != std::f32::MAX
            && minimum.z != std::f32::MAX
            && maximum.x != std::f32::MIN
            && maximum.y != std::f32::MIN
            && maximum.z != std::f32::MIN
        {
            return Some(Aabb::from_min_max(minimum, maximum));
        } else {
            None
        }
    }
}
