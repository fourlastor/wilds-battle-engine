//! Sword/Shield status rules shared by ordinary effects and Lua moves.
use crate::engine::Battle;
use crate::model::{BattleEvent, ParticipantId, Status, WeatherKind};

impl Battle {
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
            _ => false,
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
