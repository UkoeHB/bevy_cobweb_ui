use std::str::FromStr;
use std::fmt::Debug;

use bevy::{prelude::*, reflect::serde::TypedReflectDeserializer};
use serde::de::DeserializeSeed;

use serde::{Deserialize, Serialize};

#[derive(Reflect, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitStruct;

fn test_equivalence<T>(w: &World, json_raw: &str, val: T)
where
    T: Serialize + for<'de> Deserialize<'de> + Debug + Reflect + FromReflect + PartialEq
{
    // Json raw to json value
    let json_val = serde_json::Value::from_str(json_raw).unwrap();

    // Json value to reflect
    let type_registry = w.resource::<AppTypeRegistry>().read();
    let registration = type_registry.get(std::any::TypeId::of::<T>()).unwrap();
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected = deserializer.deserialize(json_val.clone()).unwrap();

    // Reflect to instruction
    let extracted = T::from_reflect(reflected.as_reflect()).unwrap();
    assert_eq!(val, extracted);

    // Val to json value
    let json_val_deser = serde_json::to_value(&val).unwrap();
    assert_eq!(json_val, json_val_deser);
}

#[test]
fn bug()
{
    let mut app = App::new();
    app.register_type::<UnitStruct>();
    test_equivalence(app.world(), "[]", UnitStruct);
}
