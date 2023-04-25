use rstar::Envelope;
use serde::ser::SerializeStruct;

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

use super::generic::pathfinding::compute_path;

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

    fn path_to_latest_objective(&self) -> Result<Option<lyon::path::Path>, PathfindError> {
        let pos = self.position();

        let component = self.pathfinding_component();
        let Some(objective) = component.objectives.front() else {  return Ok(None) };

        let path = compute_path(pos.clone(), objective, self.store_ref())
            .map_err(|_| PathfindError::EndReached(pos.clone()))
            .map(|path| path.smooth_path_2(self.store_ref()).collect::<Vec<_>>())
            .or_else(|_| {
                objective
                    .to_position(self.store_ref())
                    .map_err(|_| PathfindError::EndReached(pos.clone()))
                    .map(|obj| {
                        vec![
                            super::generic::pathfinding::PointE { x: pos.x, y: pos.y },
                            obj,
                        ]
                    })
            })?;

        Ok(Some(build_path(path)))
    }
}

pub struct TempSerEntity<'store, T: EntityRef<'store>>(pub T, std::marker::PhantomData<&'store ()>);

impl<'store, T: EntityRef<'store>> TempSerEntity<'store, T> {
    pub fn new(v: T) -> Self {
        Self(v, std::marker::PhantomData)
    }
}

impl<'store, T> serde::Serialize for TempSerEntity<'store, T>
where
    T: EntityRef<'store>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Entity", 4)?;
        s.serialize_field("guid", &self.0.guid())?;
        s.serialize_field("x", &self.0.position().x)?;
        s.serialize_field("y", &self.0.position().y)?;
        s.serialize_field("radius", &self.0.radius())?;
        s.end()
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
    fn should_unpack_parent(&self, envelope: &oobb::OOBB<f32>) -> bool {
        envelope.contains_point(&[self.0.point.x, self.0.point.y])
    }
    fn should_unpack_leaf(&self, leaf: &CollisionBox) -> bool {
        match leaf {
            CollisionBox::Polygon(_) => false,
            CollisionBox::Unit { guid, .. } => guid == &self.1,
        }
    }
}
fn build_path(
    result: Vec<impl std::borrow::Borrow<super::generic::pathfinding::PointE>>,
) -> lyon::path::Path {
    result
        .array_windows::<2>()
        .fold(
            lyon::path::Path::svg_builder(),
            |mut builder, [from, to]| {
                let from = from.borrow();
                let to = to.borrow();
                builder.move_to(lyon::math::Point::new(from.x, from.y));
                builder.line_to(lyon::math::Point::new(to.x, to.y));
                builder
            },
        )
        .build()
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



    fn pathfind_to_lastest_objective(
        &self,
        duration: crate::core::GameTimer,
    ) -> Result<Option<lyon::math::Point>, PathfindError> {
        let Some(path) = self.path_to_latest_objective()? else {return Ok(None)};
        let component = self.pathfinding_component_mut();

        let maxpos = lyon::algorithms::length::approximate_length(path.iter(), 0.1);

        let newpos = if component.position < 0.0 {
            component.position + (duration.as_secs_f32() * component.speed)
        } else {
            duration.as_secs_f32() * component.speed
        };
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
        lyon::algorithms::walk::walk_along_path(path.iter(), component.position, 0.1, &mut pattern);
        if let Some(point) = position {
            self.move_to(point);
        }
        Ok(position)
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
