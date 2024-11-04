use std::marker::PhantomData;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{ui_commands::ManagePseudoStateExt, CardinalDirection};

use super::ThemeUpdate;

pub struct AutoPseudoStatePlugin;

impl Plugin for AutoPseudoStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            propagate_flex_direction_to_pseudo_state.before(ThemeUpdate),
        )
        .add_systems(
            PostUpdate,
            propagate_visibility_to_pseudo_state
                // TODO: This is a regression that may cause a frame delay in applying the state
                // .after(VisibilitySystems::VisibilityPropagate)
                .before(ThemeUpdate),
        );
    }
}

fn propagate_flex_direction_to_pseudo_state(
    q_nodes: Query<(Entity, &Style), (With<FlexDirectionToPseudoState>, Changed<Style>)>,
    mut commands: Commands,
) {
    for (entity, style) in &q_nodes {
        if style.flex_direction == FlexDirection::Column
            || style.flex_direction == FlexDirection::ColumnReverse
        {
            commands
                .entity(entity)
                .add_pseudo_state(PseudoState::LayoutColumn);
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::LayoutRow);
        } else {
            commands
                .entity(entity)
                .add_pseudo_state(PseudoState::LayoutRow);
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::LayoutColumn);
        }
    }
}

fn propagate_visibility_to_pseudo_state(
    q_nodes: Query<
        (Entity, &Visibility, &InheritedVisibility),
        (
            With<VisibilityToPseudoState>,
            Or<(Changed<Visibility>, Changed<InheritedVisibility>)>,
        ),
    >,
    mut commands: Commands,
) {
    for (entity, visibility, inherited) in &q_nodes {
        let visible = visibility == Visibility::Visible
            || (inherited.get() && visibility == Visibility::Inherited);

        if visible {
            commands
                .entity(entity)
                .add_pseudo_state(PseudoState::Visible);
        } else {
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::Visible);
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct VisibilityToPseudoState;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FlexDirectionToPseudoState;

pub struct HierarchyToPseudoState<C>(PhantomData<C>);

impl<C> Plugin for HierarchyToPseudoState<C>
where
    C: Component,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            HierarchyToPseudoState::<C>::post_update
                .before(ThemeUpdate)
                .run_if(HierarchyToPseudoState::<C>::should_update_hierary_pseudo_states),
        );
    }
}

impl<C: Component> HierarchyToPseudoState<C> {
    pub fn new() -> Self {
        Self(PhantomData::default())
    }

    fn should_update_hierary_pseudo_states(
        q_added_tags: Query<Entity, Added<C>>,
        q_parent_changed_tags: Query<Entity, (With<C>, Changed<Parent>)>,
        q_removed_tags: RemovedComponents<C>,
    ) -> bool {
        q_added_tags.iter().count() > 0
            || q_parent_changed_tags.iter().count() > 0
            || q_removed_tags.len() > 0
    }

    fn post_update(
        q_nodes: Query<(Entity, &Parent), With<C>>,
        q_children: Query<&Children>,
        mut q_pseudo_states: Query<&mut PseudoStates>,
        mut commands: Commands,
    ) {
        let node_count = q_nodes.iter().len();
        let parents: Vec<Entity> =
            q_nodes
                .iter()
                .fold(Vec::with_capacity(node_count), |mut acc, (_, p)| {
                    if !acc.contains(&p.get()) {
                        acc.push(p.get());
                    }
                    acc
                });

        for parent in parents.iter() {
            let children = q_children.get(*parent).unwrap();
            let entities: Vec<Entity> =
                children
                    .iter()
                    .fold(Vec::with_capacity(children.len()), |mut acc, child| {
                        if q_nodes.get(*child).is_ok() {
                            acc.push(*child);
                        }

                        acc
                    });

            for (i, entity) in entities.iter().enumerate() {
                HierarchyToPseudoState::<C>::reset_hierarchy_entity_pseudo_states(
                    *entity,
                    &mut q_pseudo_states,
                    commands.reborrow(),
                );

                let mut entity_commands = commands.entity(*entity);

                if i == 0 {
                    entity_commands.add_pseudo_state(PseudoState::FirstChild);
                }
                if i == entities.len() - 1 {
                    entity_commands.add_pseudo_state(PseudoState::LastChild);
                }

                entity_commands.add_pseudo_state(PseudoState::NthChild(i));

                if i % 2 == 0 {
                    entity_commands.add_pseudo_state(PseudoState::EvenChild);
                } else {
                    entity_commands.add_pseudo_state(PseudoState::OddChild);
                }
            }

            if entities.len() == 1 {
                // Safe unwrap: length checked above
                commands
                    .entity(*entities.get(0).unwrap())
                    .add_pseudo_state(PseudoState::SingleChild);
            }
        }
    }

    fn reset_hierarchy_entity_pseudo_states(
        entity: Entity,
        q_pseudo_states: &mut Query<&mut PseudoStates>,
        mut commands: Commands,
    ) {
        if let Ok(mut pseudo_states) = q_pseudo_states.get_mut(entity) {
            let to_remove: Vec<PseudoState> = pseudo_states
                .get()
                .iter()
                .filter(|ps| match ps {
                    PseudoState::FirstChild => true,
                    PseudoState::LastChild => true,
                    PseudoState::SingleChild => true,
                    PseudoState::NthChild(_) => true,
                    PseudoState::EvenChild => true,
                    PseudoState::OddChild => true,
                    _ => false,
                })
                .map(|ps| ps.clone())
                .collect();

            for removable in to_remove {
                pseudo_states.remove(removable);
            }
        } else {
            commands.entity(entity).insert(PseudoStates::new());
        }
    }
}

#[derive(
    Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Reflect, Serialize, Deserialize,
)]
pub enum PseudoState {
    #[default]
    Enabled,
    Disabled,
    Visible,
    Selected,
    Checked,
    Empty,
    SingleChild,
    FirstChild,
    NthChild(usize),
    LastChild,
    EvenChild,
    OddChild,
    LayoutRow,
    LayoutColumn,
    OverflowX,
    OverflowY,
    Folded,
    Open,
    Closed,
    Error,
    Resizable(CardinalDirection),
    Custom(String),
}

#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct PseudoStates(Vec<PseudoState>);

impl From<Vec<PseudoState>> for PseudoStates {
    fn from(value: Vec<PseudoState>) -> Self {
        let mut uniques: Vec<PseudoState> = Vec::with_capacity(value.len());
        for val in value {
            if !uniques.contains(&val) {
                uniques.push(val);
            }
        }

        Self(uniques)
    }
}

impl PseudoStates {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn single(state: PseudoState) -> Self {
        Self(vec![state])
    }

    pub fn has(&self, state: &PseudoState) -> bool {
        self.0.contains(state)
    }

    pub fn add(&mut self, state: PseudoState) {
        if !self.0.contains(&state) {
            self.0.push(state);
        }
    }

    pub fn remove(&mut self, state: PseudoState) {
        if self.0.contains(&state) {
            // Safe unwrap: checked in if
            self.0
                .remove(self.0.iter().position(|s| *s == state).unwrap());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self) -> &Vec<PseudoState> {
        &self.0
    }
}
