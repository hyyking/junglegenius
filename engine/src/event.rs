use crate::{core::{GameTimer, Lane, Team}, structures::{TurretIndex}, wave::Wave, GameState};

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Turret(TurretIndex, TurretEvent),
    Inhibitor { team: Team, lane: Lane, event: InhibitorEvent },
    Wave { team: Team, lane: Lane, event: WaveEvent }
}

#[derive(Debug, Clone, Copy)]
pub enum TurretEvent {
    Fall,
    TakePlate,
}

#[derive(Debug, Clone, Copy)]
pub enum InhibitorEvent {
    Fall(GameTimer),
}

#[derive(Debug, Clone, Copy)]
pub enum WaveEvent {
    Spawn(Wave),
}



pub trait EventConsumer<T = Event> {
    fn on_timer_consume(&mut self, timer: GameTimer);
    fn on_event(&mut self, _event: T) {}
}

pub trait EventProducer {
    fn on_timer_produce(&self, state: &GameState) -> Box<dyn Iterator<Item = Event>>;
}