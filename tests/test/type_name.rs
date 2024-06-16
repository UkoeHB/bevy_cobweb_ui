use bevy_cobweb_ui::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(TypeName)]
struct TestType;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn test_type_name()
{
    assert_eq!("TestType", TestType::type_name());
    assert!(match "TestType"
    {
        TestType::TYPE_NAME => true,
        _ => false,
    });
    assert!(match TestType::type_name()
    {
        "TestType" => true,
        _ => false,
    });
}

//-------------------------------------------------------------------------------------------------------------------
