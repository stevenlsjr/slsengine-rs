use super::component::{ComponentManager, ComponentMask, Entity};

/// System trait.
/// Params: 'a - lifetime of component manager
pub trait EntitySystem<'a> {
    type Data;
    fn mask() -> ComponentMask;
    fn not_mask() -> ComponentMask {
        ComponentMask::NONE
    }

    /// Calback for retreiving entities based on mask from a manager
    fn make_data<I>(
        &self,
        manager: &'a ComponentManager,
        entities: I,
    ) -> Self::Data
    where
        I: Iterator<Item = Entity>;
}
