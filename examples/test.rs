use std::{env, path};

use ggez::graphics::Shader;
use ggez::input::keyboard::KeyCode;
use ggez::{
    event,
    glam::*,
    graphics::{self, Color},
    Context, GameResult,
};
use ggez_3d::canvas::DrawParam3d;
use ggez_3d::prelude::*;

struct MainState {
    canvas3d: Canvas3d,
    meshes: Vec<(Mesh3d, Vec3, Vec3)>,
    default_shader: bool,
    custom_shader: Shader,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut canvas3d = Canvas3d::new(ctx);
        let vertex_data = vec![
            // top (0.0, 0.0, 1.0)
            Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0], Color::GREEN),
            Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0], Color::GREEN),
            Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0], Color::GREEN),
            Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0], Color::new(0.0, 0.1, 0.0, 1.0)),
            // bottom (0.0, 0.0, -1.0)
            Vertex::new([-1.0, 1.0, -1.0], [1.0, 0.0], None),
            Vertex::new([1.0, 1.0, -1.0], [0.0, 0.0], None),
            Vertex::new([1.0, -1.0, -1.0], [0.0, 1.0], None),
            Vertex::new([-1.0, -1.0, -1.0], [1.0, 1.0], None),
            // right (1.0, 0.0, 0.0)
            Vertex::new([1.0, -1.0, -1.0], [0.0, 0.0], None),
            Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0], None),
            Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0], None),
            Vertex::new([1.0, -1.0, 1.0], [0.0, 1.0], None),
            // left (-1.0, 0.0, 0.0)
            Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0], None),
            Vertex::new([-1.0, 1.0, 1.0], [0.0, 0.0], None),
            Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0], None),
            Vertex::new([-1.0, -1.0, -1.0], [1.0, 1.0], None),
            // front (0.0, 1.0, 0.0)
            Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0], None),
            Vertex::new([-1.0, 1.0, -1.0], [0.0, 0.0], None),
            Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0], None),
            Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0], None),
            // back (0.0, -1.0, 0.0)
            Vertex::new([1.0, -1.0, 1.0], [0.0, 0.0], None),
            Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0], None),
            Vertex::new([-1.0, -1.0, -1.0], [1.0, 1.0], None),
            Vertex::new([1.0, -1.0, -1.0], [0.0, 1.0], None),
        ];
        let vertex_data_two = vec![
            // top (0.0, 0.0, 1.0)
            Vertex::new([2.0, 2.0, 2.0], [0.0, 0.0], None),
            Vertex::new([4.0, 2.0, 2.0], [1.0, 0.0], None),
            Vertex::new([4.0, 4.0, 2.0], [1.0, 1.0], None),
            Vertex::new([2.0, 4.0, 2.0], [0.0, 1.0], None),
            // bottom (0.0, 0.0, -1.0)
            Vertex::new([2.0, 4.0, -1.0], [1.0, 0.0], None),
            Vertex::new([4.0, 4.0, -1.0], [0.0, 0.0], None),
            Vertex::new([4.0, 2.0, -1.0], [0.0, 1.0], None),
            Vertex::new([2.0, 2.0, -1.0], [1.0, 1.0], None),
            // right (1.0, 0.0, 0.0)
            Vertex::new([4.0, 2.0, -1.0], [0.0, 0.0], None),
            Vertex::new([4.0, 4.0, -1.0], [1.0, 0.0], None),
            Vertex::new([4.0, 4.0, 2.0], [1.0, 1.0], None),
            Vertex::new([4.0, 2.0, 2.0], [0.0, 1.0], None),
            // left (-1.0, 0.0, 0.0)
            Vertex::new([2.0, 2.0, 2.0], [1.0, 0.0], None),
            Vertex::new([2.0, 4.0, 2.0], [0.0, 0.0], None),
            Vertex::new([2.0, 4.0, -1.0], [0.0, 1.0], None),
            Vertex::new([2.0, 2.0, -1.0], [1.0, 1.0], None),
            // front (0.0, 1.0, 0.0)
            Vertex::new([4.0, 4.0, -1.0], [1.0, 0.0], None),
            Vertex::new([2.0, 4.0, -1.0], [0.0, 0.0], None),
            Vertex::new([2.0, 4.0, 2.0], [0.0, 1.0], None),
            Vertex::new([4.0, 4.0, 2.0], [1.0, 1.0], None),
            // back (0.0, -1.0, 0.0)
            Vertex::new([4.0, 2.0, 2.0], [0.0, 0.0], None),
            Vertex::new([2.0, 2.0, 2.0], [1.0, 0.0], None),
            Vertex::new([2.0, 2.0, -1.0], [1.0, 1.0], None),
            Vertex::new([4.0, 2.0, -1.0], [0.0, 1.0], None),
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

        let image_two =
            graphics::Image::from_color(ctx, 1, 1, Some(graphics::Color::from_rgb(50, 10, 50)));
        let mut mesh = Mesh3d {
            vertices: vertex_data,
            indices: index_data.clone(),
            vert_buffer: None,
            ind_buffer: None,
            bind_group: None,
            texture: None,
        };

        mesh.gen_wgpu_buffer(&canvas3d.pipeline, ctx);

        let mut mesh_two = Mesh3d {
            vertices: vertex_data_two,
            indices: index_data,
            vert_buffer: None,
            ind_buffer: None,
            bind_group: None,
            texture: Some(image_two),
        };

        mesh_two.gen_wgpu_buffer(&canvas3d.pipeline, ctx);
        canvas3d.camera_bundle.camera.yaw = 90.0;
        Ok(MainState {
            canvas3d,
            meshes: vec![
                (mesh, Vec3::new(10.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0)),
                (mesh_two, Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0)),
            ],
            default_shader: true,
            custom_shader: graphics::ShaderBuilder::from_path("/fancy.wgsl")
                .build(&ctx.gfx)
                .unwrap(),
        })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) -> GameResult {
        self.canvas3d.resize(width, height, ctx);
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let k_ctx = &ctx.keyboard.clone();
        let (yaw_sin, yaw_cos) = self.canvas3d.camera_bundle.camera.yaw.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        if k_ctx.is_key_pressed(KeyCode::Q) {
            self.meshes[1].1 += 0.1;
        }
        if k_ctx.is_key_pressed(KeyCode::E) {
            self.meshes[1].1 -= 0.1;
        }
        if k_ctx.is_key_just_pressed(KeyCode::K) {
            if self.default_shader {
                self.canvas3d.set_shader(self.custom_shader.clone());
            } else {
                self.canvas3d.set_default_shader(ctx);
            }
            self.default_shader = !self.default_shader;
        }
        if k_ctx.is_key_pressed(KeyCode::Space) {
            self.canvas3d.camera_bundle.camera.position.y += 1.0;
        }
        if k_ctx.is_key_pressed(KeyCode::C) {
            self.canvas3d.camera_bundle.camera.position.y -= 1.0;
        }
        if k_ctx.is_key_pressed(KeyCode::W) {
            self.canvas3d.camera_bundle.camera.position += forward;
        }
        if k_ctx.is_key_pressed(KeyCode::S) {
            self.canvas3d.camera_bundle.camera.position -= forward;
        }
        if k_ctx.is_key_pressed(KeyCode::D) {
            self.canvas3d.camera_bundle.camera.position += right;
        }
        if k_ctx.is_key_pressed(KeyCode::A) {
            self.canvas3d.camera_bundle.camera.position -= right;
        }
        if k_ctx.is_key_pressed(KeyCode::Right) {
            self.canvas3d.camera_bundle.camera.yaw += 1.0_f32.to_radians();
        }
        if k_ctx.is_key_pressed(KeyCode::Left) {
            self.canvas3d.camera_bundle.camera.yaw -= 1.0_f32.to_radians();
        }
        if k_ctx.is_key_pressed(KeyCode::Up) {
            self.canvas3d.camera_bundle.camera.pitch += 1.0_f32.to_radians();
        }
        if k_ctx.is_key_pressed(KeyCode::Down) {
            self.canvas3d.camera_bundle.camera.pitch -= 1.0_f32.to_radians();
        }
        self.canvas3d.update_camera(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        for mesh in self.meshes.iter() {
            self.canvas3d.draw(
                mesh.0.clone(),
                DrawParam3d::default()
                    .scale(mesh.1)
                    .color(Color::new(0.5, 0.0, 0.0, 0.5)),
            );
        }
        self.canvas3d.finish(ctx, Color::BLACK)?;
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
