// Need to distinguish between CAF input and expected CAF output (after JSON round trip),
// since multi-line string formatting is lossy when entering JSON/Rust.

// Value round trip: rust type -> json value -> Caf -> json value -> reflect -> rust type
//   - Replace with CAF round trip once CAF parsing is ready. Note that Caf -> CAF -> Caf is potentially mutating
//   if whitespace is inserted during serialization.
// CAF round trip: CAF -> Caf -> json value -> reflect rust type (check against expected) -> json value
// -> Caf (+ recover fill) -> CAF
//   - Need separate sequence for testing #[reflect(default)] fields, since defaulted 'dont show' fields are not
//   known in rust.

use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitStruct;

impl ApplyLoadable for UnitStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlainStruct
{
    pub boolean: bool,
}

impl ApplyLoadable for PlainStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnumStruct
{
    #[default]
    A,
    B(UnitStruct),
    C
    {
        boolean: bool,
        s_plain: PlainStruct,
    },
}

impl ApplyLoadable for EnumStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregateStruct
{
    pub uint: u64,
    pub float: f32,
    pub boolean: bool,
    pub string: String,
    pub vec: Vec<PlainStruct>,
    pub map: HashMap<u32, u32>,
    pub s_struct: UnitStruct,
    pub s_enum: EnumStruct,
    pub s_plain: PlainStruct,
}

impl ApplyLoadable for AggregateStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WrapArray(pub Vec<UnitStruct>);

impl ApplyLoadable for WrapArray
{
    fn apply(self, _: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TupleStruct(pub UnitStruct, pub PlainStruct, pub bool);

impl ApplyLoadable for TupleStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SingleGeneric<A>
where
    A: Default + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    #[serde(skip)]
    #[reflect(ignore)]
    _p: PhantomData<A>,
}

impl<A> ApplyLoadable for SingleGeneric<A>
where
    A: Default + TypePath + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    fn apply(self, _: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SingleGenericTuple<A>(pub A);

impl<A> ApplyLoadable for SingleGenericTuple<A>
where
    A: TypePath + Loadable + Reflect + GetTypeRegistration,
{
    fn apply(self, _: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

// Test these with whitespace on the CAF value, which should be properly ignored when converting to JSON.
// MultiGeneric<uint32, bool, SingleGeneric<f32>>
// MultiGeneric<uint32, bool, (SingleGeneric<f32>, UnitStruct)>
#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiGeneric<A, B, C>
where
    A: Default + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
    B: Default + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
    C: Default + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    #[serde(skip)]
    #[reflect(ignore)]
    _p: PhantomData<(A, B, C)>,
}

impl<A, B, C> ApplyLoadable for MultiGeneric<A, B, C>
where
    A: Default + TypePath + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
    B: Default + TypePath + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
    C: Default + TypePath + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    fn apply(self, _: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnumGeneric<A>
where
    A: Default + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    A
    {
        uint: u64,

        #[serde(skip)]
        #[reflect(ignore)]
        _p: PhantomData<A>,
    },
    B
    {
        s_enum: EnumStruct,

        #[serde(skip)]
        #[reflect(ignore)]
        _p: PhantomData<A>,
    },
}

impl<A> Default for EnumGeneric<A>
where
    A: Default + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    fn default() -> Self
    {
        Self::A { uint: 0, _p: PhantomData }
    }
}

impl<A> ApplyLoadable for EnumGeneric<A>
where
    A: Default + TypePath + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    fn apply(self, _: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerdeTypesPlugin;

impl Plugin for SerdeTypesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_derived::<UnitStruct>()
            .register_derived::<PlainStruct>()
            .register_derived::<EnumStruct>()
            .register_derived::<AggregateStruct>()
            .register_derived::<WrapArray>()
            .register_derived::<TupleStruct>()
            .register_derived::<SingleGeneric<u32>>()
            .register_derived::<SingleGeneric<(u32, u32)>>()
            .register_derived::<SingleGeneric<UnitStruct>>()
            .register_derived::<SingleGeneric<SingleGeneric<u32>>>()
            .register_derived::<SingleGeneric<MultiGeneric<u32, u32, u32>>>()
            .register_derived::<SingleGenericTuple<u32>>()
            .register_derived::<SingleGenericTuple<UnitStruct>>()
            .register_derived::<SingleGenericTuple<SingleGeneric<u32>>>()
            .register_derived::<MultiGeneric<u32, u32, u32>>()
            .register_derived::<MultiGeneric<u32, u32, UnitStruct>>()
            .register_derived::<MultiGeneric<SingleGeneric<u32>, SingleGeneric<SingleGeneric<u32>>, SingleGeneric<u32>>>()
            .register_derived::<EnumGeneric<bool>>()
            .register_derived::<EnumGeneric<UnitStruct>>()
            .register_derived::<EnumGeneric<SingleGeneric<u32>>>();
    }
}

//-------------------------------------------------------------------------------------------------------------------