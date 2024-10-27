use bevy_cobweb_ui::prelude::caf::*;

use super::helpers::{test_caf, test_caf_fail};

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn scenes_section()
{
    let res = test_caf(
        b"#scenes
",
    );
    let CafSection::Scenes(scenes) = &res.sections[0] else { unreachable!() };
    assert_eq!(scenes.scenes.len(), 0);

    let res = test_caf(
        b"#scenes
\"\"
",
    );
    let CafSection::Scenes(scenes) = &res.sections[0] else { unreachable!() };
    assert_eq!(scenes.scenes.len(), 1);

    let res = test_caf(
        b"#scenes
\"a\"
\"b\"
",
    );
    let CafSection::Scenes(scenes) = &res.sections[0] else { unreachable!() };
    assert_eq!(scenes.scenes.len(), 2);

    let res = test_caf(
        b"#scenes
\"a\"
 \"b\"
",
    );
    let CafSection::Scenes(scenes) = &res.sections[0] else { unreachable!() };
    assert_eq!(scenes.scenes.len(), 1);
    assert_eq!(scenes.scenes[0].entries.len(), 1);
    let CafSceneLayerEntry::Layer(layer) = &scenes.scenes[0].entries[0] else { unreachable!() };
    assert_eq!(layer.entries.len(), 0);

    let res = test_caf(
        b"#scenes
\"a\"
    A
    \"b\"
        B
",
    );
    let CafSection::Scenes(scenes) = &res.sections[0] else { unreachable!() };
    assert_eq!(scenes.scenes.len(), 1);
    assert_eq!(scenes.scenes[0].entries.len(), 2);
    let CafSceneLayerEntry::Instruction(instruction) = &scenes.scenes[0].entries[0] else { unreachable!() };
    assert_eq!(instruction.id.to_canonical(None), "A");

    let res = test_caf(
        b"#scenes
\"a\"
    A
    \"b\"
    C<Q>::D[1 2 3]
 D{a:1}
       E(\"h\")
      \"c\"
       F
",
    );
    let CafSection::Scenes(scenes) = &res.sections[0] else { unreachable!() };
    assert_eq!(scenes.scenes.len(), 1);
    assert_eq!(scenes.scenes[0].entries.len(), 6);
    let CafSceneLayerEntry::Layer(layer) = &scenes.scenes[0].entries[5] else { unreachable!() };
    let CafSceneLayerEntry::Instruction(instruction) = &layer.entries[0] else { unreachable!() };
    assert_eq!(instruction.id.to_canonical(None), "F");
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn scenes_errors()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // Non-scene in base layer
    test_caf_fail(
        b"#scenes
\"a\"
A
",
        b"A\n",
    );
    // Non-entry in nested layer
    test_caf_fail(
        b"#scenes
\"a\"
    A
    1
",
        b"1\n",
    );
    // Entries stacked up on scene name
    test_caf_fail(
        b"#scenes
\"a\" A
",
        b"A\n",
    );
    // Entries stacked up each other
    test_caf_fail(
        b"#scenes
\"a\"
    A B
",
        b"B\n",
    );
}

//-------------------------------------------------------------------------------------------------------------------
