use specs::prelude::*;

#[derive(Debug)]
pub(crate) struct WorldSetupState {
    world_is_populated: bool,
}

impl Default for WorldSetupState {
    fn default() -> Self {
        WorldSetupState {
            world_is_populated: false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PopulateWorldSystem;

impl<'a> System<'a> for PopulateWorldSystem {
    type SystemData = (Entities<'a>, Write<'a, WorldSetupState>);

    fn run(&mut self, (entities, mut world_setup_state): Self::SystemData) {
        if world_setup_state.world_is_populated {
            return;
        }
        world_setup_state.world_is_populated = true;
    }
}
