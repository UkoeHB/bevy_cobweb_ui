use bevy_cobweb_ui::prelude::cob::*;

use super::helpers::{test_cob, test_cob_fail};

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn defs_section_scene_macros()
{
    // /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    // */
    let res = test_cob(
        b"#defs
",
    );
    let CobSection::Defs(defs) = &res.sections[0] else { unreachable!() };
    assert_eq!(defs.entries.len(), 0);

    let res = test_cob(
        b"#defs
+a = \\\\
",
    );
    let CobSection::Defs(defs) = &res.sections[0] else { unreachable!() };
    assert_eq!(defs.entries.len(), 1);
    let CobDefEntry::SceneMacro(scene_macro) = &defs.entries[0] else { unreachable!() };
    assert_eq!(scene_macro.name.as_str(), "a");
    assert_eq!(scene_macro.value.entries.len(), 0);

    let res = test_cob(
        b"#defs
+a = \\
    A
\\
",
    );
    let CobSection::Defs(defs) = &res.sections[0] else { unreachable!() };
    assert_eq!(defs.entries.len(), 1);
    let CobDefEntry::SceneMacro(scene_macro) = &defs.entries[0] else { unreachable!() };
    assert_eq!(scene_macro.name.as_str(), "a");
    let CobSceneLayerEntry::Loadable(loadable) = &scene_macro.value.entries[0] else { unreachable!() };
    assert_eq!(loadable.id.to_canonical(None), "A");

    let res = test_cob(
        b"
#defs
+a = \\
    A
\\
+b = \\
    A(10)
    -A
    ^A
    !A
\\
+c = \\
    A
    \"i\"
        B
\\
+d = \\
    +c{}
    +a{
        A
    }
\\
",
    );

    let CobSection::Defs(defs) = &res.sections[0] else { unreachable!() };
    assert_eq!(defs.entries.len(), 4);

    let CobDefEntry::SceneMacro(scene_macro) = &defs.entries[0] else { unreachable!() };
    assert_eq!(scene_macro.name.as_str(), "a");
    let CobSceneLayerEntry::Loadable(loadable) = &scene_macro.value.entries[0] else { unreachable!() };
    assert_eq!(loadable.id.to_canonical(None), "A");

    let CobDefEntry::SceneMacro(scene_macro) = &defs.entries[1] else { unreachable!() };
    assert_eq!(scene_macro.name.as_str(), "b");
    let CobSceneLayerEntry::Loadable(loadable) = &scene_macro.value.entries[0] else { unreachable!() };
    assert_eq!(loadable.id.to_canonical(None), "A");
    let CobSceneLayerEntry::SceneMacroCommand(command) = &scene_macro.value.entries[1] else { unreachable!() };
    assert_eq!(command.id.to_canonical(None), "A");
    assert_eq!(command.command_type, CobSceneMacroCommandType::Remove);
    let CobSceneLayerEntry::SceneMacroCommand(command) = &scene_macro.value.entries[2] else { unreachable!() };
    assert_eq!(command.id.to_canonical(None), "A");
    assert_eq!(command.command_type, CobSceneMacroCommandType::MoveToTop);
    let CobSceneLayerEntry::SceneMacroCommand(command) = &scene_macro.value.entries[3] else { unreachable!() };
    assert_eq!(command.id.to_canonical(None), "A");
    assert_eq!(command.command_type, CobSceneMacroCommandType::MoveToBottom);

    let CobDefEntry::SceneMacro(scene_macro) = &defs.entries[2] else { unreachable!() };
    assert_eq!(scene_macro.name.as_str(), "c");
    let CobSceneLayerEntry::Loadable(loadable) = &scene_macro.value.entries[0] else { unreachable!() };
    assert_eq!(loadable.id.to_canonical(None), "A");
    let CobSceneLayerEntry::Layer(layer) = &scene_macro.value.entries[1] else { unreachable!() };
    assert_eq!(layer.name.as_str(), "i");
    let CobSceneLayerEntry::Loadable(loadable) = &layer.entries[0] else { unreachable!() };
    assert_eq!(loadable.id.to_canonical(None), "B");

    let CobDefEntry::SceneMacro(scene_macro) = &defs.entries[3] else { unreachable!() };
    assert_eq!(scene_macro.name.as_str(), "d");
    let CobSceneLayerEntry::SceneMacroCall(call) = &scene_macro.value.entries[0] else { unreachable!() };
    assert_eq!(call.path.as_str(), "c");
    assert_eq!(call.container.entries.len(), 0);
    let CobSceneLayerEntry::SceneMacroCall(call) = &scene_macro.value.entries[1] else { unreachable!() };
    assert_eq!(call.path.as_str(), "a");
    assert_eq!(call.container.entries.len(), 1);
    let CobSceneLayerEntry::Loadable(loadable) = &call.container.entries[0] else { unreachable!() };
    assert_eq!(loadable.id.to_canonical(None), "A");
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn scene_macros_errors()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // Entry not starting with newline
    test_cob_fail(
        b"#defs
 $+ = \\\\",
        b"$+ = \\\\",
    );
    // Definition does not start with letter/number
    test_cob_fail(
        b"#defs
+_a = \\\\
",
        b"+_a = \\\\\n",
    );
    // Definition contains path segments
    test_cob_fail(
        b"#defs
+a::b = \\\\
",
        b"::b = \\\\\n",
    );
}

//-------------------------------------------------------------------------------------------------------------------
