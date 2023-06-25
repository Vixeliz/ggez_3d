use ggez::graphics::Image;
use ggez::{graphics, Context};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub fn new(p: [i8; 3], t: [i8; 2]) -> Vertex {
        Vertex {
            pos: [f32::from(p[0]), f32::from(p[1]), f32::from(p[2])],
            tex_coord: [f32::from(t[0]), f32::from(t[1])],
        }
    }
}

pub struct Mesh3d {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vert_buffer: Option<wgpu::Buffer>,
    pub ind_buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
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
                            self.texture.as_ref().unwrap().wgpu().1,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        self.bind_group = Some(bind_group);
        self.vert_buffer = Some(verts);
        self.ind_buffer = Some(inds);
    }
}
