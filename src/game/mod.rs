pub mod built_in_components;
pub mod camera;
pub mod component;
pub mod main_loop;
pub mod resource;
pub mod timer;
pub mod world;
pub use self::{camera::*, timer::*, world::*};


pub mod prelude {
    pub use super::resource::{ResourceFetcher, ResourceResult};
    pub use super::component::{Component, GetComponents};
}

pub use self::prelude::*;
/*--------------------------------------
 * Scene
 */
