pub mod camera;
pub mod mesh;
pub mod pipeline;
pub mod render;

pub mod prelude {
    pub use crate::camera::Camera;
    pub use crate::mesh::{Mesh3d, Vertex};
    pub use crate::pipeline::Pipeline3d;
}
