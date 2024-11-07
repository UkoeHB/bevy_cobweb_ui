use bevy::prelude::*;

pub trait UiContext
{
    fn get(&self, _target: &str) -> Result<Entity, String>
    {
        Err(format!(
            "{} has no UI contexts",
            std::any::type_name::<Self>()
        ))
    }

    /// These are the contexts cleared by the parent theme when no DynamicStyle
    /// is placed to them.
    ///
    /// By default this is the full list of contexts.
    ///
    /// Warning: If a context is a sub-widget with its own theme, it should not
    /// be included in the cleared contexts, nor should it be used for placement
    /// from the main entity. The behavior is undefined.
    fn cleared_contexts(&self) -> impl Iterator<Item = &str> + '_
    {
        self.contexts()
    }

    fn contexts(&self) -> impl Iterator<Item = &str> + '_
    {
        [].into_iter()
    }
}
