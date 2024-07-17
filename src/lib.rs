#[macro_use]
pub mod camera;
pub mod mesh;
pub mod renderer;
pub mod shader;
pub mod texture;
pub mod uniform;
pub mod vertex;

//Reexports
pub use bytemuck;
pub use nalgebra_glm as glm;
pub use wgpu;
pub use winit;

#[allow(unused_macros)]
macro_rules! debug {
    ($format:expr, $($expression:expr),+) => {
        #[cfg(debug_assertions)]
        println!("[{}:{}] {}", file!(), line!(), format!($format, $($expression),+))
    };
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        println!("[{}:{}] {}", file!(), line!(), $msg)
    };
}

#[allow(unused_imports)]
pub(crate) use debug;
