#![allow(dead_code)] // TODO: remove

use std::sync::Arc;

use bevy::prelude::*;
use bevy::reflect::{ApplyError, ReflectMut};

use super::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// A structure point is a specific item inside some container.
#[derive(Debug)]
pub(super) enum ReflectStructurePoint
{
    /// Includes the field name.
    Struct(&'static str),
    /// Includes the tuple index.
    TupleStruct(usize),
    /// Includes the tuple index.
    Tuple(usize),
    /// Includes the list index.
    List(usize),
    /// Includes the array index.
    Array(usize),
    /// Includes the index of the map entry.
    MapKey(usize),
    /// Includes the index of the map entry.
    MapValue(usize),
    /// Includes the index of the set entry.
    Set(usize),
    /// Includes the variant name and field index.
    Enum(&'static str, usize),
}

impl ReflectStructurePoint
{
    /// Drills into `target` to get the current point.
    fn destructure<'a>(
        &self,
        target: &'a mut (dyn PartialReflect + 'static),
    ) -> Option<&'a mut (dyn PartialReflect + 'static)>
    {
        match self {
            Self::Struct(field_name) => {
                let ReflectMut::Struct(dyn_struct) = target.reflect_mut() else { return None };
                dyn_struct.field_mut(field_name)
            }
            Self::TupleStruct(index) => {
                let ReflectMut::TupleStruct(dyn_tuplestruct) = target.reflect_mut() else { return None };
                dyn_tuplestruct.field_mut(*index)
            }
            Self::Tuple(index) => {
                let ReflectMut::Tuple(dyn_tuple) = target.reflect_mut() else { return None };
                dyn_tuple.field_mut(*index)
            }
            Self::List(index) => {
                let ReflectMut::List(dyn_list) = target.reflect_mut() else { return None };
                dyn_list.get_mut(*index)
            }
            Self::Array(index) => {
                let ReflectMut::Array(dyn_array) = target.reflect_mut() else { return None };
                dyn_array.get_mut(*index)
            }
            Self::MapKey(_index) => {
                // TODO: needs special handling, to replace a map key the previous entry needs to be removed
                // and re-inserted with the new value, but doing so will invalidate existing paths since indices
                // into the map will change
                // - can maybe be solved by using a callback instead of returning &mut PartialReflect
                None
            }
            Self::MapValue(index) => {
                let ReflectMut::Map(dyn_map) = target.reflect_mut() else { return None };
                dyn_map.get_at_mut(*index).map(|(_, value)| value)
            }
            Self::Set(_index) => {
                // TODO: needs special handling, set elements can't be edited directly
                // - can maybe be solved by using a callback instead of returning &mut PartialReflect
                None
            }
            Self::Enum(variant_name, index) => {
                let ReflectMut::Enum(dyn_enum) = target.reflect_mut() else { return None };
                if dyn_enum.variant_name() != *variant_name {
                    return None;
                }
                dyn_enum.field_at_mut(*index)
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Records a 'path' into a reflected type where a [`CobEditorWidget`] is editing.
///
/// Used to target-patch parts of a reflected type.
#[derive(Clone, Debug)]
pub(super) struct ReflectStructurePath
{
    pub(super) path: Arc<[ReflectStructurePoint]>,
}

impl ReflectStructurePath
{
    /// Finds location in target value to patch in the new value.
    ///
    /// If `path` is empty, then `value` is directly assigned to `target`. Otherwise [`PartialReflect::try_apply`]
    /// is used on an internal part of `target` pointed to by `self.path`.
    ///
    /// Returns `false` on failure.
    // TODO: if the patch value contains struct fields not present in the target because the fields are
    // reflect-defaulted (and target is dynamically reflected), then those fields *won't* be inserted to the
    // target; similarly, if some fields were removed in the patch value, they won't be removed in the
    // target
    pub(super) fn try_patch_value(
        &self,
        target: &mut Box<(dyn PartialReflect + 'static)>,
        value: Box<(dyn PartialReflect + 'static)>,
    ) -> Result<(), Option<ApplyError>>
    {
        if self.path.len() == 0 {
            *target = value;
            return Ok(());
        }

        let mut target_part = target.as_mut();

        for point in self.path.iter() {
            let Some(next) = point.destructure(target_part) else { return Err(None) };
            target_part = next;
        }

        target.try_apply(value.as_ref()).map_err(|e| Some(e))
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct CobEditorRef
{
    /// The file hash at the time this reference was created.
    ///
    /// Used to discard patches aimed at old file states.
    pub(super) file_hash: CobFileHash,
    /// Points to a scene location.
    ///
    /// If the loadable is a command, then `SceneRef::path` will equal `"#commands"`.
    pub(super) scene_ref: SceneRef,
    /// Shortname of the loadable.
    pub(super) loadable_name: &'static str,
    /// If the path is empty then the ref points to a loadable, otherwise it points to a value inside a loadable.
    pub(super) structure_path: ReflectStructurePath,
    /// Death signal used to block patch submissions if the widget has been killed by the editor.
    ///
    /// The editor kills widgets whenever a structural change is made to a destructured loadable, to ensure
    /// editor refs are recreated accurately (e.g. when inserting/removing list elements, the old widgets
    /// may have stale indices recorded).
    pub(super) death_signal: DeathSignal,
}

impl CobEditorRef
{
    /// The scene location this reference points to.
    pub fn scene_ref(&self) -> SceneRef
    {
        self.scene_ref.clone()
    }

    /// The loadable this reference points to. The reference may point to the loadable itself *or* a value inside
    /// the loadable.
    pub fn loadable_name(&self) -> &str
    {
        &self.loadable_name
    }

    /// Returns `true` if the loadable in this reference is in the `#commands` section of the target file.
    pub fn is_command(&self) -> bool
    {
        self.scene_ref.path.iter().next() == Some("#commands")
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for editor widgets that want to edit specific loadables or values that can appear in loadables.
///
/// ## Limitations
///
/// Note that if your value is *not* a loadable and contains `#[reflect(default)]` fields, widgets are currently
/// unable to add or remove such fields. Non-loadable values passed via [`SubmitPatch`] are applied with
/// [`PartialReflect::try_apply`], which only updates the intersecting fields between the original and the patch.
///
/// Loadables do not have this problem because submitted patches will directly replace the original loadable.
pub trait CobEditorWidget
{
    type Value: Reflect;

    /// Tries to spawn a widget for editing the value.
    ///
    /// The widget should be spawned as a child of the `parent` entity.
    ///
    /// You can use [`PartialReflect::clone_value`] to get an owned version of `value`, or
    /// `T::from_partial_reflect(value)` to downcast it directly (if `T` implements `FromReflect`).
    ///
    /// Note that if your value contains `#[reflect(default)]` fields, you may need to manually repair
    /// the value sent to [`SubmitPatch`] to remove fields that should remain defaulted (i.e. if you converted
    /// the `value` here to rust and then reflected it for `SubmitPatch`).
    ///
    /// Returns `false` to reject the value.
    fn try_spawn(
        c: &mut Commands,
        s: &mut SceneLoader,
        parent: Entity,
        editor_ref: &CobEditorRef,
        value: &(dyn PartialReflect + 'static),
        //settings: &CobWidgetHints  // TODO: includes 'slow_mode', etc.
    ) -> bool;
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) type EditorWidgetSpawnFn =
    fn(&mut Commands, &mut SceneLoader, Entity, &CobEditorRef, &(dyn PartialReflect + 'static)) -> bool;

//-------------------------------------------------------------------------------------------------------------------
