use legion::{
    prelude::*,
    storage::{
        ArchetypeDescription, ComponentMeta, ComponentResourceSet, ComponentTypeId,
        TagMeta, TagStorage, TagTypeId,
    },
};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use std::{cell::RefCell, any::TypeId, collections::HashMap};
use type_uuid::TypeUuid;

#[derive(TypeUuid, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[uuid = "5fd8256d-db36-4fe2-8211-c7b3446e1927"]
struct Pos(f32, f32, f32);
#[derive(TypeUuid, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[uuid = "14dec17f-ae14-40a3-8e44-e487fc423287"]
struct Vel(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct Unregistered(f32, f32, f32);

#[derive(Clone)]
struct ComponentRegistration {
    uuid: type_uuid::Bytes,
    ty: TypeId,
    tag_serialize_fn: fn(&TagStorage, &mut dyn FnMut(&dyn erased_serde::Serialize)),
    comp_serialize_fn: fn(&ComponentResourceSet, &mut dyn FnMut(&dyn erased_serde::Serialize)),
}
impl ComponentRegistration {
    fn of<T: TypeUuid + Serialize + for<'de> Deserialize<'de> + 'static>() -> Self {
        Self {
            uuid: T::UUID,
            ty: TypeId::of::<T>(),
            tag_serialize_fn: |tag_storage, serialize_fn| {
                // it's safe because we know this is the correct type due to lookup
                let slice = unsafe { tag_storage.data_slice::<T>() };
                serialize_fn(&&*slice);
            },
            comp_serialize_fn: |comp_storage, serialize_fn| {
                // it's safe because we know this is the correct type due to lookup
                let slice = unsafe { comp_storage.data_slice::<T>() };
                serialize_fn(&*slice);
            },
        }
    }
}

struct SerializeImpl {
    types: HashMap<TypeId, ComponentRegistration>,
}
impl legion::ser::WorldSerializer for SerializeImpl {
    fn can_serialize_tag(&self, ty: &TagTypeId, _meta: &TagMeta) -> bool {
        self.types.get(&ty.0).is_some()
    }
    fn can_serialize_component(&self, ty: &ComponentTypeId, _meta: &ComponentMeta) -> bool {
        self.types.get(&ty.0).is_some()
    }
    fn serialize_archetype_description<S: Serializer>(
        &self,
        serializer: S,
        archetype_desc: &ArchetypeDescription,
    ) -> Result<S::Ok, S::Error> {
        let tags_to_serialize = archetype_desc
            .tags()
            .iter()
            .filter_map(|(ty, _)| self.types.get(&ty.0))
            .map(|reg| reg.uuid)
            .collect::<Vec<_>>();
        let components_to_serialize = archetype_desc
            .components()
            .iter()
            .filter_map(|(ty, _)| self.types.get(&ty.0))
            .map(|reg| reg.uuid)
            .collect::<Vec<_>>();
        let mut desc_out = serializer.serialize_struct("ArchetypeDescription", 2)?;
        desc_out.serialize_field("tag_types", &tags_to_serialize)?;
        desc_out.serialize_field("component_types", &components_to_serialize)?;
        desc_out.end()
    }
    fn serialize_components<S: Serializer>(
        &self,
        serializer: S,
        component_type: &ComponentTypeId,
        _component_meta: &ComponentMeta,
        components: &ComponentResourceSet,
    ) -> Result<S::Ok, S::Error> {
        if let Some(reg) = self.types.get(&component_type.0) {
            let result = RefCell::new(None);
            let serializer = RefCell::new(Some(serializer));
            {
                let mut result_ref = result.borrow_mut();
                (reg.comp_serialize_fn)(components, &mut |serialize| {
                    result_ref.replace(erased_serde::serialize(serialize, serializer.borrow_mut().take().unwrap()));
                });
            }
            return result.borrow_mut().take().unwrap();
        }
        panic!("received unserializable type {:?}, this should be filtered by can_serialize", component_type);
    }
    fn serialize_tags<S: Serializer>(
        &self,
        serializer: S,
        tag_type: &TagTypeId,
        _tag_meta: &TagMeta,
        tags: &TagStorage,
    ) -> Result<S::Ok, S::Error> {
        if let Some(reg) = self.types.get(&tag_type.0) {
            let result = RefCell::new(None);
            let serializer = RefCell::new(Some(serializer));
            {
                let mut result_ref = result.borrow_mut();
                (reg.tag_serialize_fn)(tags, &mut |serialize| {
                    result_ref.replace(erased_serde::serialize(serialize, serializer.borrow_mut().take().unwrap()));
                });
            }
            return result.borrow_mut().take().unwrap();
        }
        panic!("received unserializable type {:?}, this should be filtered by can_serialize", tag_type);
    }
    fn serialize_entities<S: Serializer>(
        &self,
        serializer: S,
        entities: &[Entity],
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(entities.iter().map(|_e| *uuid::Uuid::new_v4().as_bytes() ))
    }
}

fn main() {
    // create world
    let universe = Universe::new();
    let mut world = universe.create_world();

    // Pos and Vel are both serializable, so all components in this chunkset will be serialized
    world.insert(
        (),
        vec![
            (Pos(1., 2., 3.), Vel(1., 2., 3.)),
            (Pos(1., 2., 3.), Vel(1., 2., 3.)),
            (Pos(1., 2., 3.), Vel(1., 2., 3.)),
            (Pos(1., 2., 3.), Vel(1., 2., 3.)),
        ],
    );
    // Unserializable components are not serialized, so only the Pos components should be serialized in this chunkset
    world.insert(
        (Unregistered(4., 5., 6.,),),
        vec![
            (Pos(1., 2., 3.), Unregistered(4., 5., 6.)),
            (Pos(1., 2., 3.), Unregistered(4., 5., 6.)),
            (Pos(1., 2., 3.), Unregistered(4., 5., 6.)),
            (Pos(1., 2., 3.), Unregistered(4., 5., 6.)),
        ],
    );
    // Entities with no serializable components are not serialized, so this entire chunkset should be skipped in the output
    world.insert(
        (Unregistered(4., 5., 6.,),),
        vec![
            (Unregistered(4., 5., 6.),),
            (Unregistered(4., 5., 6.),),
        ],
    );

    let registrations = [
        ComponentRegistration::of::<Pos>(),
        ComponentRegistration::of::<Vel>(),
    ];

    use std::iter::FromIterator;
    let ser_helper = SerializeImpl {
        types: HashMap::from_iter(registrations.iter().map(|reg| (reg.ty, reg.clone()))),
    };

    let serializable = legion::ser::serializable_world(&world, &ser_helper);
    println!("{}", serde_json::to_string(&serializable).unwrap());
}
