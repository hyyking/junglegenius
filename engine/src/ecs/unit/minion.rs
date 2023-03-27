use crate::{stats::{GoldCollectable, UnitStatistics, WithUnitStats}, core::Team, ecs::Unit};



#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MinionType {
    Melee,
    Ranged,
    Siege,
    SuperMinion,
}

impl MinionType {
    pub(crate) fn radius(&self) -> f32 {
        match self {
            MinionType::Melee => 48.0,
            MinionType::Ranged => 48.0,
            MinionType::Siege => 65.0,
            MinionType::SuperMinion => 65.0,
        }
    }
}

impl GoldCollectable for MinionType {
    fn golds(&self) -> usize {
        0
    }
}
#[derive(Clone, Copy)]
pub struct Minion {
    pub team: Team,
    pub ty: MinionType,
    pub upgrades: usize,
    pub ms_upgrades: usize,
    pub mid_debuf: bool,
    pub position: Option<lyon::math::Point>,
}

impl std::fmt::Debug for Minion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self.ty {
            MinionType::Melee => "Melee",
            MinionType::Ranged => "Ranged",
            MinionType::Siege => "Siege",
            MinionType::SuperMinion => "SuperMinion",
        };
        f.debug_struct(name)
            .field("golds", &self.golds())
            .field("ms", &self.movespeed())
            .finish()
    }
}

impl Minion {
    pub const MELEE_GOLD: usize = 21;
    pub const RANGED_GOLD: usize = 14;
    pub const BIG_GOLD: usize = 57;

    pub fn movespeed(&self) -> usize {
        325 + 25 * self.ms_upgrades
    }
}

impl Unit for Minion {
    fn team(&self) -> Team {
        self.team
    }
    fn position(&self) -> lyon::math::Point {
        self.position.unwrap_or_default()
    }

    fn radius(&self) -> f32 {
        self.ty.radius() 
    }
}

impl GoldCollectable for Minion {
    fn golds(&self) -> usize {
        match self.ty {
            MinionType::Melee => Self::MELEE_GOLD - usize::from(self.mid_debuf),
            MinionType::Ranged => Self::RANGED_GOLD - usize::from(self.mid_debuf),
            MinionType::Siege | MinionType::SuperMinion => {
                std::cmp::min(Self::BIG_GOLD + (3 * self.upgrades), 90)
                    - usize::from(self.mid_debuf)
            }
        }
    }

    fn to_last_hit(&self) -> usize {
        1
    }
}

impl WithUnitStats for Minion {
    fn base_stats(&self) -> UnitStatistics {
        let mut stats = UnitStatistics::default();
        stats.movespeed = self.movespeed() as f32;
        match self.ty {
            MinionType::Melee => {
                stats.health = 455.0;
                stats.attack_speed = 1.25;
                stats.attack_damage = 12.0;
                stats.range = 110.0;
            }
            MinionType::Ranged => {
                stats.health = 290.0;
                stats.attack_speed = 0.667;
                stats.attack_damage = 22.5;
                stats.range = 550.0;
            }
            MinionType::Siege => {
                stats.health = 850.0;
                stats.attack_speed = 1.0;
                stats.attack_damage = 41.0;
                stats.range = 300.0;
            }
            MinionType::SuperMinion => {
                stats.health = 1600.0;
                stats.attack_damage = 230.0;
                stats.health_regen = 67.5 / 5.0;
                stats.attack_speed = 0.85;
                stats.range = 170.0;
                stats.armor = 100.0;
                stats.magic_resist = -30.0;
            }
        }
        stats
    }

    fn current_stats(&self, _: &crate::core::GameTimer) -> UnitStatistics {
        let mut stats = self.base_stats();
        match self.ty {
            MinionType::Melee => {
                // https://leagueoflegends.fandom.com/wiki/Melee_minion
                let upgrades = (self.upgrades as f32 + 1.0).min(25.0);
                stats.health = if upgrades <= 5.0 {
                    stats.health + 22.0 * upgrades + 0.3 * (upgrades - 1.0) / 2.0 * upgrades
                } else {
                    stats.health
                        + 22.0 * 5.0
                        + 32.25 * (upgrades - 5.0)
                        + 0.3 * (upgrades - 1.0) / 2.0 * upgrades
                };
                stats.attack_damage = if upgrades <= 5.0 {
                    stats.attack_damage
                } else {
                    stats.attack_damage + 3.41 * (upgrades - 5.0)
                };
            }
            MinionType::Ranged => {
                // https://leagueoflegends.fandom.com/wiki/Caster_minion
                let upgrades = (self.upgrades as f32 + 1.0).min(25.0);
                stats.health = if upgrades <= 5.0 {
                    stats.health + 6.0 * upgrades
                } else {
                    stats.health + 6.0 * 5.0 + 8.25 * (upgrades - 5.0)
                };
                stats.attack_damage = if upgrades <= 5.0 {
                    stats.attack_damage + 1.5 * upgrades
                } else {
                    stats.attack_damage + 1.5 * 5.0 + 4.5 * (upgrades - 5.0)
                };
            }
            MinionType::Siege => {
                // https://leagueoflegends.fandom.com/wiki/Siege_minion
                let upgrades = (self.upgrades as f32 + 1.0).min(117.0);
                stats.health = if upgrades <= 5.0 {
                    stats.health + 62.0 * upgrades
                } else {
                    stats.health + 62.0 * 5.0 + 87.0 * (upgrades - 5.0)
                };
                stats.attack_damage = stats.attack_damage + 1.5 * (self.upgrades as f32).min(6667.0);
            }
            MinionType::SuperMinion => {
                // https://leagueoflegends.fandom.com/wiki/Super_minion
                let upgrades = (self.upgrades as f32 + 1.0).min(100.0);
                stats.health = stats.health + 100.0 * upgrades;

                stats.attack_damage = stats.attack_damage + 5.0 * (self.upgrades as f32).min(200.0);

                stats.health_regen = if upgrades <= 8.0 {
                    stats.health_regen
                } else {
                    stats.health_regen + 0.775 * (upgrades.min(25.0) - 5.0)
                };
            }
        }
        stats
    }
}
