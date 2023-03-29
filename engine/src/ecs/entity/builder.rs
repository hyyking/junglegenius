use crate::{ecs::{UnitId, generic::{PositionComponent, PathfindingComponent}}, structures::{turret::TurretComponent, inhibitor::InhibitorComponent}, units::minion::MinionComponent};


pub enum SpecificComponentBuilder {
    None,
    Turret(TurretComponent),
    Inhibitor(InhibitorComponent),
    Minion(MinionComponent),
}


pub trait EntityBuilder {
    fn guid(&self) -> UnitId;
    fn position(&self) -> PositionComponent;
    fn pathfinding(&self)  -> PathfindingComponent;
    fn specific(&self) -> SpecificComponentBuilder;
}
