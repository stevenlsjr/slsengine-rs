pub mod camera;
pub mod main_loop;
pub mod resource;
pub mod timer;
pub mod world;

pub use self::{
    camera::*,
    timer::*,
    world::{InputSources, InputState, WorldManager},
};
pub mod prelude {
    pub use super::main_loop::{FrameTick, MainLoopState};
    pub use super::resource::{ResourceFetcher, ResourceResult};
}

pub use self::prelude::*;
/*--------------------------------------
 * Scene
 */
