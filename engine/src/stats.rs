use crate::core::GameTimer;

pub trait WithUnitStats {
    fn base_stats(&self) -> UnitStatistics;

    fn current_stats(&self, _: &GameTimer) -> UnitStatistics {
        self.base_stats()
    }
}

pub trait GoldCollectable {
    fn golds(&self) -> usize;
    fn to_last_hit(&self) -> usize {
        0
    }
}

pub trait GoldCollectableIterator<T>: Iterator<Item = T>
where
    T: GoldCollectable,
{
    fn collect_last_hits(self) -> usize
    where
        Self: Sized,
    {
        self.map(|c| c.to_last_hit()).sum()
    }

    fn collect_golds(self) -> usize
    where
        Self: Sized,
    {
        self.map(|c| c.golds()).sum()
    }
}

impl<T, U> GoldCollectableIterator<T> for U
where
    U: Iterator<Item = T>,
    T: GoldCollectable,
{
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UnitStatistics {
    /* Offensive Stats */
    pub ability_power: f32,
    pub armor_penetration: f32,
    pub attack_damage: f32,
    pub attack_speed: f32,
    pub critical_strike_chance: f32,
    pub critical_strike_damage: f32,
    pub lifesteal: f32,
    pub magic_penetration: f32,
    pub omnivamp: f32,
    pub physicalvamp: f32,

    /* Defensive Stats */
    pub armor: f32,
    pub heal_shield_power: f32,
    pub health: f32,
    pub health_regen: f32,
    pub magic_resist: f32,
    pub tenacity: f32,
    pub slow_resists: f32,

    /* Utility Stats */
    pub ability_haste: f32,
    pub energy: f32,
    pub energy_regen: f32,
    pub mana: f32,
    pub mana_regen: f32,

    /* Other Stats */
    pub experience: f32,
    pub gold_generation: f32,
    pub movespeed: f32,
    pub range: f32,
}
