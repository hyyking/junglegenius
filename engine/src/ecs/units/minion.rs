use crate::{
    core::{Lane, Team},
    ecs::{
        self,
        entity::{Entity, EntityBuilder, EntityMut, EntityRef, SpecificComponentBuilder, EntityRefCrateExt},
        generic::{
            pathfinding::{PathfindingComponent, LANE_PATHS},
            PositionComponent,
        },
        store::EntityStore,
    },
};

use super::old_minion::MinionType;

#[derive(Debug, Clone)]
pub struct MinionComponent {
    pub kind: crate::units::old_minion::MinionType,
}

pub struct Minion<'store> {
    pub(crate) store: &'store crate::ecs::store::EntityStore,
    pub(crate) entity: &'store Entity,
}

impl<'store> EntityRef<'store> for Minion<'store> {
    fn store_ref(&self) -> &'store EntityStore {
        self.store
    }
    fn entity(&self) -> &'store Entity {
        self.entity
    }
}

pub struct MinionMut<'store> {
    pub(crate) store: &'store mut crate::ecs::store::EntityStore,
    pub(crate) entity: std::ptr::NonNull<Entity>,
}

impl MinionMut<'_> {
    pub fn get_state(&self) -> &MinionComponent {
        &self.store.minions[self.get_specific_unchecked().unwrap()].1
    }
}

impl<'store> EntityRef<'store> for MinionMut<'store> {
    fn store_ref(&self) -> &'store EntityStore {
        unsafe { &*(self.store as *const _) }
    }
    fn entity(&self) -> &'store Entity {
        unsafe { self.entity.as_ref() }
    }
}

impl<'store> EntityMut<'store> for MinionMut<'store> {
    fn store_mut(&self) -> &'store mut EntityStore {
        unsafe { &mut *(self.store as *const _ as *mut _) }
    }
}

impl<'a> std::fmt::Debug for MinionMut<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Turret")
            .field("id", &self.entity().guid)
            .field("position", &self.position())
            .field("state", self.get_state())
            .finish()
    }
}

#[derive(Default)]
pub struct MinionBuilder {
    kind: Option<MinionType>,
    lane: Option<Lane>,
    team: Option<Team>,
    offset: f32,
}

impl MinionBuilder {
    pub fn ranged() -> Self {
        Self {
            kind: Some(MinionType::Ranged),
            ..Default::default()
        }
    }
    pub fn melee() -> Self {
        Self {
            kind: Some(MinionType::Melee),
            ..Default::default()
        }
    }
    pub fn siege() -> Self {
        Self {
            kind: Some(MinionType::Siege),
            ..Default::default()
        }
    }
    pub fn superm() -> Self {
        Self {
            kind: Some(MinionType::SuperMinion),
            ..Default::default()
        }
    }

    pub fn set_lane(mut self, lane: Lane) -> Self {
        self.lane = Some(lane);
        self
    }

    pub fn set_team(mut self, team: Team) -> Self {
        self.team = Some(team);
        self
    }

    pub fn set_offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    fn kind(&self) -> MinionType {
        self.kind.expect("minion kind was not set")
    }

    fn team(&self) -> Team {
        self.team.expect("minion team was not set")
    }

    fn lane(&self) -> Lane {
        self.lane.expect("minion lane was not set")
    }

    fn path(&self) -> &std::sync::Arc<lyon::path::Path> {
        &LANE_PATHS[(self.team(), self.lane())]
    }

    fn radius(&self) -> f32 {
        match self.kind() {
            MinionType::Melee => 48.0,
            MinionType::Ranged => 48.0,
            MinionType::Siege => 65.0,
            MinionType::SuperMinion => 65.0,
        }
    }
}

impl EntityBuilder for MinionBuilder {
    fn guid(&self) -> ecs::UnitId {
        ecs::UnitId::new(Some(self.team()), Some(self.lane()))
    }

    fn position(&self) -> PositionComponent {
        PositionComponent {
            point: self.path().first_endpoint().unwrap().0,
            radius: self.radius(),
        }
    }

    fn pathfinding(&self) -> PathfindingComponent {
        // TODO: Dynamic speed
        PathfindingComponent::persistent(std::sync::Arc::clone(self.path()), 325.0)
            .offset_position(self.offset)
    }

    fn specific(&self) -> SpecificComponentBuilder {
        SpecificComponentBuilder::Minion(MinionComponent { kind: self.kind() })
    }
}
