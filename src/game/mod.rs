pub mod camera;
pub mod components;
pub mod main_loop;
pub(crate) mod populate_world_system;
pub mod resource;
pub mod timer;
pub mod world;

pub use self::{
    camera::*,
    timer::*,
    world::{InputSources, InputState, WorldManager},
};
pub mod prelude {
    pub use super::components::*;
    pub use super::main_loop::{FrameTick, MainLoopState};
    pub use super::resource;
}

pub use self::prelude::*;
/*--------------------------------------
 * Scene
 */
