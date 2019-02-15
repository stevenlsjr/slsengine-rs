pub mod built_in_components;
pub mod camera;
pub mod component;
pub mod component_stores;
pub mod main_loop;
pub mod resource;
pub mod system;
pub mod timer;
pub mod world;

pub use self::{
    camera::*,
    timer::*,
    world::{EntityWorld, InputSources, InputState},
};
pub mod prelude {
    pub use super::component::Component;
    pub use super::component_stores::{GetComponent, Storage, TryGetComponent};
    pub use super::main_loop::{FrameTick, MainLoopState};
    pub use super::resource::{ResourceFetcher, ResourceResult};
    pub use super::system::EntitySystem;
}

pub use self::prelude::*;
/*--------------------------------------
 * Scene
 */
