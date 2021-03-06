use std::error::Error;

use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use error::NoError;
use saveload::marker::Marker;
use shred::SystemData;
use storage::{ReadStorage, WriteStorage};
use world::{Component, Entity};

#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct EntityData<M: Marker, E, T: Components<M::Identifier, E>> {
    pub marker: M,
    pub components: T::Data,
}

/// This trait should be implemented in order to allow component
/// to be serializable with `SerializeSystem`.
/// It is automatically implemented for all
/// `Component + DeserializeOwned + Serialize + Copy`
pub trait SaveLoadComponent<M>: Component {
    /// Serializable data representation for component
    type Data: Serialize + DeserializeOwned;

    /// Error may occur during serialization or deserialization of component
    type Error: Error;

    /// Convert this component into serializable form (`Data`) using
    /// entity to marker mapping function
    fn save<F>(&self, ids: F) -> Result<Self::Data, Self::Error>
    where
        F: FnMut(Entity) -> Option<M>;

    /// Convert this component into deserializable form (`Data`) using
    /// marker to entity mapping function
    fn load<F>(data: Self::Data, ids: F) -> Result<Self, Self::Error>
    where
        F: FnMut(M) -> Option<Entity>;
}

impl<C, M> SaveLoadComponent<M> for C
where
    C: Component + DeserializeOwned + Serialize + Copy,
{
    type Data = Self;
    type Error = NoError;

    fn save<F>(&self, _ids: F) -> Result<Self::Data, NoError> {
        Ok(*self)
    }

    fn load<F>(data: Self, _ids: F) -> Result<Self, NoError> {
        Ok(data)
    }
}

/// Helper trait defines storages tuples for components tuple.
pub trait Storages<'a> {
    /// Storages for read
    type ReadStorages: SystemData<'a> + 'a;
    /// Storages for write
    type WriteStorages: SystemData<'a> + 'a;
}

/// This trait is implemented for all tuples where all elements are
/// `Component + Serialize + DeserializeOwned`.
pub trait Components<M, E>: for<'a> Storages<'a> {
    /// Serializable and deserializable intermediate representation
    type Data: Serialize + DeserializeOwned;

    /// Saves `Component`s of entity into `Data` serializable representation
    fn save<'a, F>(
        entity: Entity,
        storages: &<Self as Storages<'a>>::ReadStorages,
        ids: F,
    ) -> Result<Self::Data, E>
    where
        F: FnMut(Entity) -> Option<M>;

    /// Loads `Component`s to entity from `Data` deserializable representation
    fn load<'a, F>(
        entity: Entity,
        components: Self::Data,
        storages: &mut <Self as Storages<'a>>::WriteStorages,
        ids: F,
    ) -> Result<(), E>
    where
        F: FnMut(M) -> Option<Entity>;
}

macro_rules! impl_components {
    ($($a:ident|$b:ident),*) => {
        impl<'a, $($a),*> Storages<'a> for ($($a,)*)
            where $(
                $a: Component,
            )*
        {
            type ReadStorages = ($(ReadStorage<'a, $a>,)*);
            type WriteStorages = ($(WriteStorage<'a, $a>,)*);
        }

        impl<M, E $(,$a)*> Components<M, E> for ($($a,)*)
            where $(
                $a: SaveLoadComponent<M>,
                E: From<$a::Error>,
            )*
        {
            type Data = ($(Option<$a::Data>,)*);

            #[allow(unused_variables, unused_mut, non_snake_case)]
            fn save<'a, F>(entity: Entity, storages: &($(ReadStorage<'a, $a>,)*), mut ids: F)
                -> Result<($(Option<$a::Data>,)*), E>
                where F: FnMut(Entity) -> Option<M>
            {
                let ($(ref $b,)*) = *storages;
                Ok(($(
                    $b.get(entity).map(|c| c.save(&mut ids).map(Some)).unwrap_or(Ok(None))?,
                )*))
            }

            #[allow(unused_variables, unused_mut, non_snake_case)]
            fn load<'a, F>(entity: Entity, components: ($(Option<$a::Data>,)*),
                           storages: &mut ($(WriteStorage<'a, $a>,)*), mut ids: F)
                -> Result<(), E>
                where F: FnMut(M) -> Option<Entity>
            {
                let ($($a,)*) = components;
                let ($(ref mut $b,)*) = *storages;
                $(
                    if let Some(a) = $a {
                        $b.insert(entity, $a::load(a, &mut ids)?);
                    } else {
                        $b.remove(entity);
                    }
                )*
                Ok(())
            }
        }

        // Recursively implement for smaller tuple
        impl_components!(@ $($a|$b),*);
    };

    // List depleted. End of recursion
    (@) => {};

    // Cut head of the list and call macro again
    (@ $ah:ident|$bh:ident $(,$a:ident|$b:ident)*) => {
        // Call again for tail
        impl_components!($($a|$b),*);
    };
}

impl_components!(
    LA | LB,
    MA | MB,
    NA | NB,
    OA | OB,
    PA | PB,
    QA | QB,
    RA | RB,
    SA | SB,
    TA | TB,
    UA | UB,
    VA | VB,
    WA | WB,
    XA | XB,
    YA | YB,
    ZA | ZB
);
