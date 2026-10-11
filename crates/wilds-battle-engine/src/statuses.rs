//! Sword/Shield status rules shared by ordinary effects and Lua moves.
use crate::engine::Battle;
use crate::model::{BattleEvent, ParticipantId, Rule, Status, WeatherKind};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusCause {
    Direct,
    Hazard,
    Synchronize,
}

impl Battle {
    pub(crate) fn team_status_ability(&self, who: ParticipantId, ability: &str) -> bool {
        self.ignored_ability != Some(who)
            && self
                .actors()
                .into_iter()
                .any(|ally| ally.side == who.side && self.has_ability(ally, ability))
    }

    pub(crate) fn status_sun_active(&self) -> bool {
        self.status_weather_active(WeatherKind::Sun)
    }

    pub(crate) fn status_weather_active(&self, kind: WeatherKind) -> bool {
        self.effective_weather() == Some(kind)
    }

    pub(crate) fn status_ability_immunity(&self, who: ParticipantId, status: Status) -> bool {
        match status {
            Status::Frozen => self.has_ability(who, "magmaarmor"),
            Status::Burned => {
                self.has_ability(who, "waterveil") || self.has_ability(who, "waterbubble")
            }
            Status::Poisoned | Status::BadlyPoisoned => {
                self.has_ability(who, "immunity") || self.has_ability(who, "pastelveil")
            }
            Status::Paralyzed => self.has_ability(who, "limber"),
            Status::Asleep => {
                self.has_ability(who, "insomnia") || self.has_ability(who, "vitalspirit")
            }
        }
    }

    pub(crate) fn pastel_veil_entry(&mut self, events: &mut Vec<BattleEvent>) {
        for who in self.actors() {
            if matches!(
                self.get(who).status,
                Some(Status::Poisoned | Status::BadlyPoisoned)
            ) && self.team_status_ability(who, "pastelveil")
            {
                self.cure_status(who, None, events);
            }
        }
    }

    pub(crate) fn residual_status_cures(&mut self, events: &mut Vec<BattleEvent>) {
        let mut order = self.actors();
        order.sort_by_key(|&who| std::cmp::Reverse(self.action_speed(who)));
        for who in order {
            if self.get(who).status.is_some()
                && ((self.has_ability(who, "hydration")
                    && self.status_weather_active(WeatherKind::Rain)
                    && self.weather.as_ref().is_some_and(|w| w.turns_left > 0))
                    || (self.has_ability(who, "shedskin") && self.rng.chance(0.33)))
            {
                self.cure_status(who, None, events);
            }
            if self.has_ability(who, "healer") {
                for ally in self.actors() {
                    if who.side == ally.side
                        && self.adjacent(who, ally)
                        && self.get(ally).status.is_some()
                        && self.rng.chance(0.3)
                    {
                        self.cure_status(ally, None, events);
                    }
                }
            }
        }
    }

    pub(crate) fn poison_residual(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        let p = self.get(who);
        let toxic = p.status == Some(Status::BadlyPoisoned);
        let stage = u16::from(p.status_turns.clamp(1, 15));
        let max_hp = p.max_hp;
        let amount = if toxic {
            (max_hp / 16).max(1) * stage
        } else {
            (max_hp / 8).max(1)
        };
        if toxic {
            self.get_mut(who).status_turns = (stage as u8 + 1).min(15);
        }
        if self.has_ability(who, "poisonheal") {
            self.heal(who, (max_hp / 8).max(1), events);
        } else if !self.has_ability(who, "magicguard") {
            events.push(BattleEvent::Message(format!(
                "{} is hurt by poison!",
                self.get(who).name
            )));
            self.damage(who, amount, events);
        }
    }

    pub(crate) fn update_status_abilities(&mut self, events: &mut Vec<BattleEvent>) {
        for who in self.actors() {
            if self.has_ability(who, "oblivious") {
                self.get_mut(who).conditions.retain(|c| {
                    c.name != "taunt" && !c.rules.iter().any(|r| matches!(r, Rule::InfatuatedBy(_)))
                });
            }
            if self.has_ability(who, "owntempo") {
                self.get_mut(who).confused_turns = None;
            }
            if self.get(who).status.is_some_and(|status| {
                self.status_ability_immunity(who, status)
                    || (status == Status::Asleep && self.field_prevents_sleep())
            }) {
                self.cure_status(who, None, events);
            }
        }
    }

    pub(crate) fn can_infatuate(&self, source: ParticipantId, target: ParticipantId) -> bool {
        use crate::model::Gender::{Female, Male};
        self.get(source).hp > 0
            && self.get(target).hp > 0
            && self.is_active(source)
            && self.is_active(target)
            && matches!(
                (self.get(source).gender, self.get(target).gender),
                (Male, Female) | (Female, Male)
            )
            && !self.has_ability(target, "oblivious")
            && !self.team_status_ability(target, "aromaveil")
    }

    pub(crate) fn field_prevents_sleep(&self) -> bool {
        self.field_conditions.iter().flat_map(|c| &c.rules).any(|r|
            matches!(r, Rule::BlockStatus {statuses, ..} if statuses.is_empty() || statuses.contains(&Status::Asleep)))
    }

    pub(crate) fn is_asleep(&self, who: ParticipantId) -> bool {
        self.get(who).status == Some(Status::Asleep) || self.has_ability(who, "comatose")
    }

    pub(crate) fn confusion_damage(&mut self, who: ParticipantId) -> u16 {
        use crate::model::CombatStat;
        let p = self.get(who);
        let attack = p.effective_stat(CombatStat::Attack) as u64;
        let defense = self.battle_staged_stat(who, CombatStat::Defense).max(1.0) as u64;
        // Confusion uses stages but no burn, Guts, STAB, type or critical modifiers.
        let base = ((2 * u64::from(p.level) / 5 + 2) * 40 * attack / defense / 50 + 2) as u16;
        (f32::from(base) * self.rng.damage_roll()).floor().max(1.0) as u16
    }

    pub(crate) fn bad_dreams_residual(&mut self, events: &mut Vec<BattleEvent>) {
        for source in self.actors() {
            if !self.has_ability(source, "baddreams") {
                continue;
            }
            for target in self.actors() {
                if source.side != target.side
                    && self.is_asleep(target)
                    && !self.has_ability(target, "magicguard")
                {
                    self.damage(target, (self.get(target).max_hp / 8).max(1), events);
                }
            }
        }
    }

    pub(crate) fn thaw(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        if self.get(who).hp > 0 && self.cure_status(who, Some(Status::Frozen), events) {
            events.push(BattleEvent::Message(format!(
                "{} thawed out!",
                self.get(who).name
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CombatStat, Condition, MoveCatalog, Pokemon, Rule, Side};

    #[test]
    fn paralysis_rounds_after_speed_multipliers_and_quick_feet_uses_any_status() {
        let mut user = Pokemon::new("User", vec!["splash".into()]);
        user.speed = 201;
        user.status = Some(Status::Paralyzed);
        user.conditions.push(Condition {
            name: "tailwind".into(),
            value: 1,
            turns_left: None,
            rules: vec![Rule::StatMultiplier {
                stat: CombatStat::Speed,
                factor: 2.0,
            }],
            group: None,
            end_message: None,
        });
        let target = Pokemon::new("Target", vec!["splash".into()]);
        let mut battle = Battle::new(
            1,
            [vec![user], vec![target]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        let who = ParticipantId {
            side: Side::Allies,
            index: 0,
        };
        assert_eq!(battle.stat_value(who, CombatStat::Speed), 201.0);
        battle.get_mut(who).ability = Some("quick-feet".into());
        assert_eq!(battle.stat_value(who, CombatStat::Speed), 603.0);
        battle.get_mut(who).status = Some(Status::Burned);
        assert_eq!(battle.stat_value(who, CombatStat::Speed), 603.0);
    }
}
