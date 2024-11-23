// Need to distinguish between COB input and expected COB output (after JSON round trip),
// since multi-line string formatting is lossy when entering JSON/Rust.

// Value round trip: rust type -> json value -> Cob -> json value -> reflect -> rust type
//   - Replace with COB round trip once COB parsing is ready. Note that Cob -> COB -> Cob is potentially mutating
//   if whitespace is inserted during serialization.
// COB round trip: COB -> Cob -> json value -> reflect rust type (check against expected) -> json value
// -> Cob (+ recover fill) -> COB
//   - Need separate sequence for testing #[reflect(default)] fields, since defaulted 'dont show' fields are not
//   known in rust.

use std::collections::BTreeMap;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::reflect::{GetTypeRegistration, Typed};
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitStruct;

impl Instruction for UnitStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlainStruct
{
    pub boolean: bool,
}

impl Instruction for PlainStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloatStruct(pub f64);

impl Instruction for FloatStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringStruct(pub String);

impl Instruction for StringStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleTupleStruct(pub u32, pub u32);

impl Instruction for SimpleTupleStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleStruct
{
    pub a: u32,
    pub b: u32,
}

impl Instruction for SimpleStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewtypeStruct<T>(pub T);

impl<T> Instruction for NewtypeStruct<T>
where
    T: Typed + Loadable + Reflect + GetTypeRegistration,
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WrapNewtypeStruct(pub NewtypeStruct<u32>);

impl Instruction for WrapNewtypeStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NewtypeEnum
{
    Tuple(()),
    #[default]
    X,
}

impl Instruction for NewtypeEnum
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainsNewtypes
{
    pub n: WrapNewtypeStruct,
    pub w: WrapArray,
}

impl Instruction for ContainsNewtypes
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
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
    D(SimpleTupleStruct),
    E(SimpleStruct),
}

impl Instruction for EnumStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
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
    pub map: BTreeMap<u32, u32>, /* TODO: use map that preserves insertion ordering (that implements
                                  * reflect/serialize/deserialize) */
    pub s_struct: UnitStruct,
    pub s_enum: EnumStruct,
    pub s_plain: PlainStruct,
}

impl Instruction for AggregateStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WrapArray(pub Vec<UnitStruct>);

impl Instruction for WrapArray
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TupleStruct(pub UnitStruct, pub PlainStruct, pub bool);

impl Instruction for TupleStruct
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
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

impl<A> Instruction for SingleGeneric<A>
where
    A: Default + TypePath + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SingleGenericTuple<A>(pub A);

impl<A> Instruction for SingleGenericTuple<A>
where
    A: Typed + Loadable + GetTypeRegistration,
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

// Test these with whitespace on the COB value, which should be properly ignored when converting to JSON.
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

impl<A, B, C> Instruction for MultiGeneric<A, B, C>
where
    A: Default + TypePath + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
    B: Default + TypePath + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
    C: Default + TypePath + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
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

impl<A> Instruction for EnumGeneric<A>
where
    A: Default + TypePath + std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuiltinColor(pub Color);

impl Instruction for BuiltinColor
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuiltinCollection
{
    pub auto_val: Val,
    pub px: Val,
    pub percent: Val,
    pub vw: Val,
    pub vh: Val,
    pub vmin: Val,
    pub vmax: Val,
    pub color: Color,
}

impl Instruction for BuiltinCollection
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectDefaulted
{
    #[reflect(default)]
    pub a: u32,
    #[reflect(default)]
    pub b: u32,
}

impl Instruction for ReflectDefaulted
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct SerdeReflectDefaulted
{
    #[reflect(default = "SerdeReflectDefaulted::default_a")]
    #[serde(default = "SerdeReflectDefaulted::default_a")]
    pub a: u32,
    #[reflect(default = "SerdeReflectDefaulted::default_b")]
    #[serde(default = "SerdeReflectDefaulted::default_b")]
    pub b: u32,
}

impl SerdeReflectDefaulted
{
    fn default_a() -> u32
    {
        10
    }
    fn default_b() -> u32
    {
        20
    }
}

impl Instruction for SerdeReflectDefaulted
{
    fn apply(self, _: Entity, _: &mut World) {}
    fn revert(_: Entity, _: &mut World) {}
}

impl Default for SerdeReflectDefaulted
{
    fn default() -> Self
    {
        Self { a: Self::default_a(), b: Self::default_b() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerdeTypesPlugin;

impl Plugin for SerdeTypesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_instruction_type::<UnitStruct>()
            .register_instruction_type::<PlainStruct>()
            .register_instruction_type::<FloatStruct>()
            .register_instruction_type::<StringStruct>()
            .register_instruction_type::<SimpleTupleStruct>()
            .register_instruction_type::<SimpleStruct>()
            .register_instruction_type::<NewtypeStruct<()>>()
            .register_instruction_type::<NewtypeStruct<u32>>()
            .register_instruction_type::<NewtypeStruct<(u32, u32)>>()
            .register_instruction_type::<NewtypeStruct<UnitStruct>>()
            .register_instruction_type::<NewtypeStruct<NewtypeStruct<UnitStruct>>>()
            .register_instruction_type::<NewtypeStruct<SimpleTupleStruct>>()
            .register_instruction_type::<NewtypeStruct<SimpleStruct>>()
            .register_instruction_type::<NewtypeStruct<NewtypeStruct<u32>>>()
            .register_instruction_type::<WrapNewtypeStruct>()
            .register_instruction_type::<NewtypeEnum>()
            .register_instruction_type::<ContainsNewtypes>()
            .register_instruction_type::<EnumStruct>()
            .register_instruction_type::<AggregateStruct>()
            .register_instruction_type::<WrapArray>()
            .register_instruction_type::<TupleStruct>()
            .register_instruction_type::<SingleGeneric<u32>>()
            .register_instruction_type::<SingleGeneric<(u32, u32)>>()
            .register_instruction_type::<SingleGeneric<UnitStruct>>()
            .register_instruction_type::<SingleGeneric<SingleGeneric<u32>>>()
            .register_instruction_type::<SingleGeneric<MultiGeneric<u32, u32, u32>>>()
            .register_instruction_type::<SingleGenericTuple<u32>>()
            .register_instruction_type::<SingleGenericTuple<UnitStruct>>()
            .register_instruction_type::<SingleGenericTuple<SingleGeneric<u32>>>()
            .register_instruction_type::<MultiGeneric<u32, u32, u32>>()
            .register_instruction_type::<MultiGeneric<u32, u32, UnitStruct>>()
            .register_instruction_type::<MultiGeneric<SingleGeneric<u32>, SingleGeneric<SingleGeneric<u32>>, SingleGeneric<u32>>>()
            .register_instruction_type::<EnumGeneric<bool>>()
            .register_instruction_type::<EnumGeneric<UnitStruct>>()
            .register_instruction_type::<EnumGeneric<SingleGeneric<u32>>>()
            .register_instruction_type::<BuiltinColor>()
            .register_instruction_type::<BuiltinCollection>()
            .register_instruction_type::<ReflectDefaulted>()
            .register_instruction_type::<SerdeReflectDefaulted>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
