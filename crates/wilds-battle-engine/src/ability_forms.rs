//! Ability forms use canonical form slugs and stats supplied by the host in `Pokemon::forms`.
use crate::engine::Battle;
use crate::items::item_key;
use crate::model::{BattleEvent, ParticipantId, WeatherKind};

impl Battle {
    pub(crate) fn residual_ability_forms(&mut self, events: &mut Vec<BattleEvent>) {
        for who in self.actors() {
            self.health_ability_form(who, false, events);
        }
    }

    pub(crate) fn health_ability_form(
        &mut self,
        who: ParticipantId,
        entry: bool,
        events: &mut Vec<BattleEvent>,
    ) {
        if self.get(who).transformed() {
            return;
        }
        if !entry && self.has_ability(who, "zenmode") && self.species_is(who, 555, "darmanitan") {
            let galar = self
                .get(who)
                .form
                .as_deref()
                .is_some_and(|f| f.contains("galar"));
            let (base, zen) = if galar {
                ("darmanitan-galar-standard", "darmanitan-galar-zen")
            } else {
                ("darmanitan-standard", "darmanitan-zen")
            };
            let form = if self.get(who).hp <= self.get(who).max_hp / 2 {
                zen
            } else {
                base
            };
            self.ability_form(who, form, base, events);
        }
    }

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
        if self.species_is(who, 421, "cherrim")
            && self
                .get(who)
                .ability
                .as_deref()
                .is_some_and(|id| item_key(id) == "flowergift")
        {
            let form = if self.has_ability(who, "flowergift")
                && self.effective_weather() == Some(WeatherKind::Sun)
            {
                "cherrim-sunshine"
            } else {
                "cherrim-overcast"
            };
            self.ability_form(who, form, "cherrim-overcast", events);
        }
    }

    pub(crate) fn team_ability_stat_factor(
        &self,
        who: ParticipantId,
        stat: crate::model::CombatStat,
    ) -> f32 {
        use crate::model::CombatStat;
        if !matches!(stat, CombatStat::Attack | CombatStat::SpDefense)
            || (stat == CombatStat::SpDefense && self.ability_bypass)
            || self.effective_weather() != Some(WeatherKind::Sun)
        {
            return 1.0;
        }
        self.actors()
            .into_iter()
            .filter(|&source| {
                source.side == who.side
                    && self.species_is(source, 421, "cherrim")
                    && self.has_ability(source, "flowergift")
            })
            .fold(1.0, |factor, _| factor * 1.5)
    }
}
