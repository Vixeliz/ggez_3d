use std::{env, path};

use ggez::input::keyboard::KeyCode;
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};
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

        let image = graphics::Image::from_color(
            ctx,
            1,
            1,
            Some(graphics::Color::from_rgb(0x20, 0xA0, 0xC0)),
        );
        let image_two =
            graphics::Image::from_color(ctx, 1, 1, Some(graphics::Color::from_rgb(50, 10, 50)));
        let mut mesh = Mesh3d {
            vertices: vertex_data,
            indices: index_data.clone(),
            vert_buffer: None,
            ind_buffer: None,
            bind_group: None,
            texture: Some(image),
        };

        mesh.gen_wgpu_buffer(&pipeline3d.pipeline, ctx);

        let mut mesh_two = Mesh3d {
            vertices: vertex_data_two,
            indices: index_data,
            vert_buffer: None,
            ind_buffer: None,
            bind_group: None,
            texture: Some(image_two),
        };

        mesh_two.gen_wgpu_buffer(&pipeline3d.pipeline, ctx);
        pipeline3d.meshes = vec![mesh, mesh_two];
        pipeline3d.camera_bundle.camera.yaw = 90.0;
        Ok(MainState { pipeline3d })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.pipeline3d.resize(width, height, ctx);
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let k_ctx = &ctx.keyboard;
        let (yaw_sin, yaw_cos) = self.pipeline3d.camera_bundle.camera.yaw.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        if k_ctx.is_key_pressed(KeyCode::W) {
            self.pipeline3d.camera_bundle.camera.position += forward;
        }
        if k_ctx.is_key_pressed(KeyCode::S) {
            self.pipeline3d.camera_bundle.camera.position -= forward;
        }
        if k_ctx.is_key_pressed(KeyCode::D) {
            self.pipeline3d.camera_bundle.camera.position += right;
        }
        if k_ctx.is_key_pressed(KeyCode::A) {
            self.pipeline3d.camera_bundle.camera.position -= right;
        }
        if k_ctx.is_key_pressed(KeyCode::Right) {
            self.pipeline3d.camera_bundle.camera.yaw += 1.0_f32.to_radians();
        }
        if k_ctx.is_key_pressed(KeyCode::Left) {
            self.pipeline3d.camera_bundle.camera.yaw -= 1.0_f32.to_radians();
        }
        if k_ctx.is_key_pressed(KeyCode::Up) {
            self.pipeline3d.camera_bundle.camera.pitch += 1.0_f32.to_radians();
        }
        if k_ctx.is_key_pressed(KeyCode::Down) {
            self.pipeline3d.camera_bundle.camera.pitch -= 1.0_f32.to_radians();
        }
        self.pipeline3d.update_camera(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.pipeline3d.draw(ctx, Color::from_rgb(25, 25, 25));
        let mut canvas = graphics::Canvas::from_frame(ctx, None);

        // Do ggez drawing
        let dest_point1 = Vec2::new(10.0, 210.0);
        let dest_point2 = Vec2::new(10.0, 250.0);
        canvas.draw(
            &graphics::Text::new("You can mix ggez and wgpu drawing;"),
            dest_point1,
        );
        canvas.draw(
            &graphics::Text::new("it basically draws wgpu stuff first, then ggez"),
            dest_point2,
        );

        canvas.finish(ctx)?;

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
