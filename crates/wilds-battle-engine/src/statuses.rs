//! Sword/Shield status rules shared by ordinary effects and Lua moves.
use crate::engine::Battle;
use crate::model::{BattleEvent, ParticipantId, Status, WeatherKind};

impl Battle {
    pub(crate) fn team_status_ability(&self, who: ParticipantId, ability: &str) -> bool {
        self.ignored_ability != Some(who)
            && self
                .actors()
                .into_iter()
                .any(|ally| ally.side == who.side && self.has_ability(ally, ability))
    }

    pub(crate) fn status_sun_active(&self) -> bool {
        self.weather
            .as_ref()
            .is_some_and(|w| w.kind == WeatherKind::Sun)
            && !self
                .actors()
                .into_iter()
                .any(|who| self.has_ability(who, "cloudnine") || self.has_ability(who, "airlock"))
    }

    pub(crate) fn status_ability_immunity(&self, who: ParticipantId, status: Status) -> bool {
        match status {
            Status::Frozen => self.has_ability(who, "magmaarmor"),
            Status::Burned => {
                self.has_ability(who, "waterveil") || self.has_ability(who, "waterbubble")
            }
            Status::Poisoned | Status::BadlyPoisoned => {
                self.has_ability(who, "immunity") || self.team_status_ability(who, "pastelveil")
            }
            Status::Paralyzed => self.has_ability(who, "limber"),
            _ => false,
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
            if self
                .get(who)
                .status
                .is_some_and(|status| self.status_ability_immunity(who, status))
            {
                self.cure_status(who, None, events);
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
