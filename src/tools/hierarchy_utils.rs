use bevy::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Searches for a component in the ancestors of an entity.
///
/// Does not look at the entity itself for the component.
pub fn get_ancestor_mut<T: Component>(world: &mut World, entity: Entity) -> Option<(Entity, &mut T)>
{
    get_ancestor_mut_filtered(world, entity, |_| true)
}

//-------------------------------------------------------------------------------------------------------------------

/// Searches for a component in the ancestors of an entity.
///
/// Filters encountered components with the given callback.
///
/// Does not look at the entity itself for the component.
pub fn get_ancestor_mut_filtered<T: Component>(
    world: &mut World,
    entity: Entity,
    filter: impl Fn(&T) -> bool,
) -> Option<(Entity, &mut T)>
{
    let mut current = entity;
    let mut found = false;
    while let Some(parent) = world.get::<Parent>(current) {
        current = parent.get();
        let Some(component) = world.get::<T>(current) else { continue };
        if !(filter)(&component) {
            continue;
        }
        found = true;
        break;
    }

    // Outside loop due to borrow checker limitations.
    if found {
        let component = world.get_mut::<T>(current).unwrap();
        return Some((current, component.into_inner()));
    }

    None
}

//-------------------------------------------------------------------------------------------------------------------

/// Iterates descendants of an entity, applying the given callback.
///
/// Will stop descending if the `filter` returns `false`.
pub fn iter_descendants_filtered(
    world: &World,
    entity: Entity,
    filter: impl Fn(&World, Entity) -> bool,
    mut callback: impl FnMut(&World, Entity),
)
{
    fn iter_impl(
        world: &World,
        entity: Entity,
        filter: &impl Fn(&World, Entity) -> bool,
        callback: &mut impl FnMut(&World, Entity),
    )
    {
        // Filter
        if !(*filter)(world, entity) {
            return;
        }

        // Callback
        (*callback)(world, entity);

        // Iterate into children.
        if let Some(children) = world.get::<Children>(entity) {
            for child in children.iter() {
                iter_impl(world, *child, filter, callback);
            }
        }
    }

    iter_impl(world, entity, &filter, &mut callback)
}

//-------------------------------------------------------------------------------------------------------------------

/// More performant than `.iter_descendants`, which internally allocates every time you call it.
#[derive(Resource, Default)]
pub struct IterChildren
{
    stack: Vec<Entity>,
}

impl IterChildren
{
    pub fn search<R>(
        &mut self,
        initial_entity: Entity,
        children_query: &Query<&Children>,
        search_fn: impl FnMut(Entity) -> Option<R>,
    ) -> Option<R>
    {
        self.stack.clear();
        self.stack.push(initial_entity);
        self.search_impl(children_query, search_fn)
    }

    pub fn search_descendants<R>(
        &mut self,
        initial_children: &Children,
        children_query: &Query<&Children>,
        search_fn: impl FnMut(Entity) -> Option<R>,
    ) -> Option<R>
    {
        self.stack.clear();
        self.stack.extend(initial_children);
        self.search_impl(children_query, search_fn)
    }

    pub fn search_impl<R>(
        &mut self,
        children_query: &Query<&Children>,
        mut search_fn: impl FnMut(Entity) -> Option<R>,
    ) -> Option<R>
    {
        while let Some(entity) = self.stack.pop() {
            if let Some(entry) = (search_fn)(entity) {
                return Some(entry);
            }

            if let Ok(next_children) = children_query.get(entity) {
                self.stack.extend(next_children);
            }
        }

        None
    }
}

//-------------------------------------------------------------------------------------------------------------------
