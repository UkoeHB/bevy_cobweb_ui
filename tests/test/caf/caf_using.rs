use bevy_cobweb_ui::prelude::caf::*;

use super::helpers::{test_caf, test_caf_fail};

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn using_section()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let res = test_caf(
        b"#using
",
    );
    let CafSection::Using(using) = &res.sections[0] else { unreachable!() };
    assert_eq!(using.entries.len(), 0);

    let res = test_caf(
        b"#using
A as A
",
    );
    let CafSection::Using(using) = &res.sections[0] else { unreachable!() };
    assert_eq!(using.entries.len(), 1);
    assert_eq!(using.entries[0].type_path.to_canonical(None), "A");
    assert_eq!(using.entries[0].identifier.to_canonical(None), "A");

    let res = test_caf(
        b"#using
a::b::A as A
",
    );
    let CafSection::Using(using) = &res.sections[0] else { unreachable!() };
    assert_eq!(using.entries.len(), 1);
    assert_eq!(using.entries[0].type_path.to_canonical(None), "a::b::A");
    assert_eq!(using.entries[0].identifier.to_canonical(None), "A");

    let res = test_caf(
        b"#using
A<u32, B, C<D>> as A<u32, B, C<D>>
",
    );
    let CafSection::Using(using) = &res.sections[0] else { unreachable!() };
    assert_eq!(using.entries.len(), 1);
    assert_eq!(using.entries[0].type_path.to_canonical(None), "A<u32, B, C<D>>");
    assert_eq!(using.entries[0].identifier.to_canonical(None), "A<u32, B, C<D>>");

    let res = test_caf(
        b"
#using
A as B
a::A as B
a::b::A<B> as C
",
    );
    let CafSection::Using(using) = &res.sections[0] else { unreachable!() };
    assert_eq!(using.entries.len(), 3);
    assert_eq!(using.entries[0].type_path.to_canonical(None), "A");
    assert_eq!(using.entries[0].identifier.to_canonical(None), "B");
    assert_eq!(using.entries[1].type_path.to_canonical(None), "a::A");
    assert_eq!(using.entries[1].identifier.to_canonical(None), "B");
    assert_eq!(using.entries[2].type_path.to_canonical(None), "a::b::A<B>");
    assert_eq!(using.entries[2].identifier.to_canonical(None), "C");
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn using_errors()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // Non-entry in section
    test_caf_fail(
        b"#using
A as B
1",
        b"1",
    );
    // Entry not starting with newline
    test_caf_fail(
        b"#using
 A as B",
        b"A as B",
    );
    // No fill after 'as'
    test_caf_fail(
        b"#using
A asB",
        b"B",
    );
    // No fill before 'as'
    test_caf_fail(
        b"#using
Aas B",
        b"B",
    );
    // Path not lowercase
    test_caf_fail(
        b"#using
A::A as B",
        b"::A as B",
    );
    // Type not uppercase
    test_caf_fail(
        b"#using
a::a as B",
        b"a::a as B",
    );
    // Alias not uppercase
    test_caf_fail(
        b"#using
A as b",
        b"b",
    );
    // Generics in path not resolved
    test_caf_fail(
        b"#using
A<@a> as B",
        b"A<@a> as B",
    );
    // Generics in alias not resolved
    test_caf_fail(
        b"#using
A as B<@b>",
        b"@b>",
    );
    // Path starting with '::'
    test_caf_fail(
        b"#using
::A as B",
        b"::A as B",
    );
    // Path not interspersed with single '::'
    test_caf_fail(
        b"#using
a:::b::A as B",
        b"a:::b::A as B",
    );
}

//-------------------------------------------------------------------------------------------------------------------
