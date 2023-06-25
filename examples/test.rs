use std::{env, path};

use ggez::{event, graphics, Context, GameResult};
use ggez_3d::prelude::*;

struct MainState {
    pipeline3d: Pipeline3d,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut pipeline3d = Pipeline3d::new(ctx);
        let vertex_data = vec![
            // top (0, 0, 1)
            Vertex::new([-1, -1, 1], [0, 0]),
            Vertex::new([1, -1, 1], [1, 0]),
            Vertex::new([1, 1, 1], [1, 1]),
            Vertex::new([-1, 1, 1], [0, 1]),
            // bottom (0, 0, -1)
            Vertex::new([-1, 1, -1], [1, 0]),
            Vertex::new([1, 1, -1], [0, 0]),
            Vertex::new([1, -1, -1], [0, 1]),
            Vertex::new([-1, -1, -1], [1, 1]),
            // right (1, 0, 0)
            Vertex::new([1, -1, -1], [0, 0]),
            Vertex::new([1, 1, -1], [1, 0]),
            Vertex::new([1, 1, 1], [1, 1]),
            Vertex::new([1, -1, 1], [0, 1]),
            // left (-1, 0, 0)
            Vertex::new([-1, -1, 1], [1, 0]),
            Vertex::new([-1, 1, 1], [0, 0]),
            Vertex::new([-1, 1, -1], [0, 1]),
            Vertex::new([-1, -1, -1], [1, 1]),
            // front (0, 1, 0)
            Vertex::new([1, 1, -1], [1, 0]),
            Vertex::new([-1, 1, -1], [0, 0]),
            Vertex::new([-1, 1, 1], [0, 1]),
            Vertex::new([1, 1, 1], [1, 1]),
            // back (0, -1, 0)
            Vertex::new([1, -1, 1], [0, 0]),
            Vertex::new([-1, -1, 1], [1, 0]),
            Vertex::new([-1, -1, -1], [1, 1]),
            Vertex::new([1, -1, -1], [0, 1]),
        ];
        let vertex_data_two = vec![
            // top (0, 0, 1)
            Vertex::new([2, 2, 2], [0, 0]),
            Vertex::new([4, 2, 2], [1, 0]),
            Vertex::new([4, 4, 2], [1, 1]),
            Vertex::new([2, 4, 2], [0, 1]),
            // bottom (0, 0, -1)
            Vertex::new([2, 4, -1], [1, 0]),
            Vertex::new([4, 4, -1], [0, 0]),
            Vertex::new([4, 2, -1], [0, 1]),
            Vertex::new([2, 2, -1], [1, 1]),
            // right (1, 0, 0)
            Vertex::new([4, 2, -1], [0, 0]),
            Vertex::new([4, 4, -1], [1, 0]),
            Vertex::new([4, 4, 2], [1, 1]),
            Vertex::new([4, 2, 2], [0, 1]),
            // left (-1, 0, 0)
            Vertex::new([2, 2, 2], [1, 0]),
            Vertex::new([2, 4, 2], [0, 0]),
            Vertex::new([2, 4, -1], [0, 1]),
            Vertex::new([2, 2, -1], [1, 1]),
            // front (0, 1, 0)
            Vertex::new([4, 4, -1], [1, 0]),
            Vertex::new([2, 4, -1], [0, 0]),
            Vertex::new([2, 4, 2], [0, 1]),
            Vertex::new([4, 4, 2], [1, 1]),
            // back (0, -1, 0)
            Vertex::new([4, 2, 2], [0, 0]),
            Vertex::new([2, 2, 2], [1, 0]),
            Vertex::new([2, 2, -1], [1, 1]),
            Vertex::new([4, 2, -1], [0, 1]),
        ];

        #[rustfmt::skip]
        let index_data: Vec<u32> = vec![
             0,  1,  2,  2,  3,  0, // top
             4,  5,  6,  6,  7,  4, // bottom
             8,  9, 10, 10, 11,  8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        let image =
            graphics::Image::from_solid(ctx, 1, graphics::Color::from_rgb(0x20, 0xA0, 0xC0));
        let image_two = graphics::Image::from_solid(ctx, 1, graphics::Color::from_rgb(50, 10, 50));
        let mut mesh = Mesh3d {
            vertices: vertex_data,
            indices: index_data.clone(),
            vert_buffer: None,
            ind_buffer: None,
            bind_group: None,
            texture: Some(image),
        };

        mesh.gen_wgpu_buffer(&pipeline3d.pipeline, ctx);
        pipeline3d.meshes = vec![mesh];
        Ok(MainState { pipeline3d })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.pipeline3d.resize(width, height, ctx);
        // println!("Resized screen to {}, {}", width, height);
        // self.camera.aspect = width / height;
        // self.camera_uniform.update_view_proj(&self.camera);
        // ctx.gfx.wgpu().queue.write_buffer(
        //     &self.camera_buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.camera_uniform]),
        // );
        // let new_rect = graphics::Rect::new(0.0, 0.0, width as f32, height as f32);
        // self.screen_coords = new_rect;
        Ok(())
    }

    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.pipeline3d.draw(ctx);
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("cube", "ggez")
        .window_mode(ggez::conf::WindowMode::default().resizable(true))
        .add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}
