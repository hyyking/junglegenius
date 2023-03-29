use crate::{
    ecs::{
        generic::{pathfinding::PathfindingComponent, PositionComponent},
        UnitId,
    },
    structures::{inhibitor::InhibitorComponent, turret::TurretComponent},
    units::minion::MinionComponent,
};

pub enum SpecificComponentBuilder {
    None,
    Turret(TurretComponent),
    Inhibitor(InhibitorComponent),
    Minion(MinionComponent),
}

pub trait EntityBuilder {
    fn guid(&self) -> UnitId;
    fn position(&self) -> PositionComponent;
    fn pathfinding(&self) -> PathfindingComponent;
    fn specific(&self) -> SpecificComponentBuilder;
}
