use super::component::{ComponentManager, ComponentMask ,Entity};
use super::component_stores::TryGetComponent;
#[derive(Debug)]
pub enum SystemDispatch {
    Update,
    FixedUpdate,
    Once,
    Never,
}

/// System trait.
/// Params: 'a - lifetime of component manager
pub trait EntitySystem<'a, ComponentStore> where ComponentStore: TryGetComponent {
    const DISPATCH: SystemDispatch = SystemDispatch::Never;
    type Data: Clone + Send + Sync;

    fn dispatch(&self, manager: &mut ComponentManager<ComponentStore>, data: Self::Data);

    /// Calback for retreiving entities based on mask from a manager
    fn prep_data<I>(
        &self,
        manager: &'a ComponentManager<ComponentStore>,
        entities: I,
    ) -> Result<Self::Data, failure::Error>
    where
        I: Iterator<Item = Entity> + 'a;
}

// #[test]
// fn test_entity_system() {
//     use super::{component::*, component_stores::*};

//     #[derive(Debug, Copy, Clone, PartialEq)]
//     struct DummyComponent(u32);
//     impl Component for DummyComponent {}
//     #[derive(Debug, Clone)]
//     struct SpawnSystem(usize);
//     impl<'a> EntitySystem<'a> for SpawnSystem {
//         const DISPATCH: SystemDispatch = SystemDispatch::Once;

//         type Data = Arc<Storage<DummyComponent>>;
//         fn prep_data<I>(
//             &self,
//             manager: &'a ComponentManager,
//             entities: I,
//         ) -> Result<Self::Data, failure::Error>
//         where
//             I: Iterator<Item = Entity> + 'a,
//         {
//             let dummies = manager
//                 .other_components::<DummyComponent>()
//                 .ok_or(format_err!("no dummy component store"))?;
//             Ok((dummies))
//         }

//         fn dispatch(&self, manager: &mut ComponentManager, data: Self::Data) {
//             let mut entities: Vec<Entity> = Vec::with_capacity(self.0);
//             let mut dummies = data.0.write().unwrap();
//             for i in 0..self.0 {
//                 let e = manager.alloc_entity();
//                 dbg!((i, e));
//                 dummies.insert(*e, DummyComponent(i as u32));
//                 assert_eq!(dummies[*e], Some(DummyComponent(i as u32)))
//             }
//         }
//     }

//     let mut mgr = ComponentManager::new();
//     mgr.register::<DummyComponent>();
//     let sys = SpawnSystem(10);
//     {
//         let data = sys
//             .prep_data(&mgr, mgr.entities())
//             .expect("should be able to prep data");
//         sys.dispatch(&mut mgr, data);
//     }

//     {
//         let cmp = mgr.other_components::<DummyComponent>().unwrap();
//         let reader = cmp.read().unwrap();
//         for e in mgr.entities() {
//             dbg!((e, reader[*e]));
//         }
//     }

//     assert_eq!(mgr.entities().count(), sys.0);
//     assert_eq!(
//         mgr.entities()
//             .filter(|&e| {
//                 let dummies = mgr.other_components::<DummyComponent>().unwrap();
//                 dummies.read().map(|store| store[*e].is_some()).unwrap()
//             })
//             .count(),
//         sys.0
//     );
// }
