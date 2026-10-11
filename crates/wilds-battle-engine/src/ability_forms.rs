//! Ability forms use canonical form slugs and stats supplied by the host in `Pokemon::forms`.
use crate::engine::Battle;
use crate::items::item_key;
use crate::model::{BattleEvent, ParticipantId, WeatherKind};

impl Battle {
    pub(crate) fn species_is(&self, who: ParticipantId, id: u32, name: &str) -> bool {
        self.get(who).species_id == id || item_key(&self.get(who).species) == name
    }

    fn ability_form(
        &mut self,
        who: ParticipantId,
        form: &str,
        base: &str,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if self.get(who).hp == 0 || self.get(who).transformed() {
            return false;
        }
        if form == base
            && self
                .get(who)
                .battle_base_form
                .as_ref()
                .is_some_and(|(id, _)| id.is_none())
        {
            return self.restore_form(who, events);
        }
        self.change_form(who, form, events)
    }

    pub(crate) fn update_ability_form(
        &mut self,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) {
        if self.get(who).transformed() {
            return;
        }
        if self.species_is(who, 351, "castform")
            && self
                .get(who)
                .ability
                .as_deref()
                .is_some_and(|id| item_key(id) == "forecast")
        {
            let form = match self
                .effective_weather()
                .filter(|_| self.has_ability(who, "forecast"))
            {
                Some(WeatherKind::Sun) => "castform-sunny",
                Some(WeatherKind::Rain) => "castform-rainy",
                Some(WeatherKind::Hail) => "castform-snowy",
                _ => "castform",
            };
            self.ability_form(who, form, "castform", events);
        }
    }
}
