use rstar::{Envelope};

use crate::{
    core::Team,
    ecs::{
        generic::{
            pathfinding::{PathfindError, PathfindingComponent},
            PositionComponent,
        },
        store::EntityStore,
        UnitId,
    },
    nav_engine::CollisionBox,
};

mod builder;
pub use builder::{EntityBuilder, SpecificComponentBuilder};

#[derive(Debug, Clone)]
pub enum SpecificComponent {
    None,
    Turret(usize),
    Inhibitor(usize),
    Minion(usize),
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub guid: UnitId,
    pub position: usize,
    pub pathfinding: usize,
    pub specific: SpecificComponent,
}

impl Entity {
    pub fn is_turret(&self) -> bool {
        matches!(self.specific, SpecificComponent::Turret(_))
    }

    pub fn is_inhib(&self) -> bool {
        matches!(self.specific, SpecificComponent::Inhibitor(_))
    }
    pub(crate) fn is_minion(&self) -> bool {
        matches!(self.specific, SpecificComponent::Minion(_))
    }
}

pub trait EntityRef<'store> {
    fn store_ref(&self) -> &'store EntityStore;
    fn entity(&self) -> &Entity;

    fn guid(&self) -> UnitId {
        self.entity().guid
    }

    fn team(&self) -> Option<Team> {
        self.entity().guid.team()
    }

    fn position(&self) -> &'store lyon::math::Point {
        &self.position_component().point
    }

    fn radius(&self) -> f32 {
        self.position_component().radius
    }
}

pub(crate) trait EntityRefCrateExt<'store>: EntityRef<'store> {
    fn position_component(&self) -> &'store PositionComponent {
        &self.store_ref().position[self.entity().position].1
    }

    fn pathfinding_component(&self) -> &'store PathfindingComponent {
        &self.store_ref().pathfinding[self.entity().pathfinding].1
    }

    fn get_specific_unchecked(&self) -> Option<usize> {
        match self.entity().specific {
            SpecificComponent::Turret(a) => Some(a),
            SpecificComponent::Inhibitor(a) => Some(a),
            SpecificComponent::Minion(a) => Some(a),
            SpecificComponent::None => None,
        }
    }
}

pub(crate) struct UnitRemoval(pub(crate) PositionComponent, pub(crate) UnitId);

impl rstar::SelectionFunction<CollisionBox> for UnitRemoval {
    fn should_unpack_parent(&self, envelope: &oobb::OOBB) -> bool {
        envelope.contains_point(&[self.0.point.x, self.0.point.y])
    }
    fn should_unpack_leaf(&self, leaf: &CollisionBox) -> bool {
        match leaf {
            CollisionBox::Polygon(_) => false,
            CollisionBox::Unit { guid, .. } => guid == &self.1,
        }
    }
}

pub trait EntityMut<'store>: EntityRef<'store> {
    fn store_mut(&self) -> &'store mut EntityStore;

    fn move_to(&self, to: lyon::math::Point) {
        let store = self.store_mut();

        let to = PositionComponent {
            point: to,
            ..*self.position_component()
        };
        let prev = std::mem::replace(self.position_component_mut(), to);

        store
            .nav
            .tree
            .remove_with_selection_function(UnitRemoval(prev, self.guid()));
        store.nav.tree.insert(CollisionBox::Unit {
            position: to,
            guid: self.guid(),
        });
    }

    fn pathfind_for_duration(
        &self,
        duration: crate::core::GameTimer,
    ) -> Result<Option<lyon::math::Point>, PathfindError> {
        let component = self.pathfinding_component_mut();

        match &component.path {
            super::generic::pathfinding::Pathfinding::Static => Ok(None),
            super::generic::pathfinding::Pathfinding::Persistent(path) => {
                let maxpos = lyon::algorithms::length::approximate_length(path.iter(), 0.1);

                let newpos = component.position + (duration.as_secs_f32() * component.speed);
                if newpos >= maxpos {
                    let point = path.last_endpoint().unwrap().0;
                    self.move_to(point);
                    return Err(PathfindError::EndReached(point));
                }
                component.position = newpos;

                let mut position = None;
                let mut pattern = lyon::algorithms::walk::RegularPattern {
                    callback: &mut |event: lyon::algorithms::walk::WalkerEvent| {
                        position = Some(lyon::math::Point::new(event.position.x, event.position.y));
                        false
                    },
                    interval: component.speed as f32,
                };
                lyon::algorithms::walk::walk_along_path(
                    path.iter(),
                    component.position,
                    0.1,
                    &mut pattern,
                );
                if let Some(point) = position {
                    self.move_to(point);
                }
                Ok(position)
            }
            super::generic::pathfinding::Pathfinding::Dynamic {
                path: _path,
                start: _start,
                end: _end,
            } => {
                unimplemented!(
                    "maybe change start/end to a set duration after which no pathfinding is done"
                )
            }
        }
    }

    fn delete(self) -> Result<UnitId, String>
    where
        Self: Sized,
    {
        self.store_mut().remove_by_id(self.guid())
    }
}

pub(crate) trait EntityMutCrateExt<'store>: EntityMut<'store> {
    fn position_component_mut(&self) -> &'store mut PositionComponent {
        &mut self.store_mut().position[self.entity().position].1
    }

    fn pathfinding_component_mut(&self) -> &'store mut PathfindingComponent {
        &mut self.store_mut().pathfinding[self.entity().pathfinding].1
    }
}
impl<'store, T> EntityRefCrateExt<'store> for T where T: EntityRef<'store> + ?Sized {}
impl<'store, T> EntityMutCrateExt<'store> for T where T: EntityMut<'store> + ?Sized {}
