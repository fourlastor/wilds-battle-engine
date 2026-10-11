//! Ability forms use canonical form slugs and stats supplied by the host in `Pokemon::forms`.
use crate::engine::Battle;
use crate::items::item_key;
use crate::model::{BattleEvent, ParticipantId, WeatherKind};

impl Battle {
    fn update_mimicry(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        use crate::model::{PokemonType, Scope};
        if !self.has_ability(who, "mimicry") {
            self.end_condition(Scope::Pokemon(who), "mimicry");
            return;
        }
        let terrain = self
            .field_conditions
            .iter()
            .find_map(|c| match c.name.as_str() {
                "electric_terrain" => Some((1, PokemonType::Electric)),
                "grassy_terrain" => Some((2, PokemonType::Grass)),
                "misty_terrain" => Some((3, PokemonType::Fairy)),
                "psychic_terrain" => Some((4, PokemonType::Psychic)),
                _ => None,
            });
        let value = terrain.map_or(0, |(n, _)| n);
        if self
            .get(who)
            .conditions
            .iter()
            .any(|c| c.name == "mimicry" && c.value == value)
        {
            return;
        }
        self.set_mark(Scope::Pokemon(who), "mimicry", value, None);
        let types = terrain.map_or_else(|| self.get(who).original_types(), |(_, kind)| vec![kind]);
        if self.types_of(who) != types {
            self.get_mut(who).types = types;
            self.get_mut(who).added_type = None;
            events.push(BattleEvent::Message(format!(
                "{} changed type with Mimicry!",
                self.get(who).name
            )));
        }
    }

    pub(crate) fn shields_down_blocks_status(&self, who: ParticipantId) -> bool {
        self.has_ability(who, "shieldsdown")
            && self.species_is(who, 774, "minior")
            && !self.get(who).transformed()
            && self
                .get(who)
                .form
                .as_deref()
                .is_some_and(|id| id.contains("meteor"))
    }
    pub(crate) fn stance_before_move(
        &mut self,
        who: ParticipantId,
        spec: &crate::moves::MoveSpec,
        events: &mut Vec<BattleEvent>,
    ) {
        if self.has_ability(who, "stancechange") && self.species_is(who, 681, "aegislash") {
            let shield = item_key(&spec.id) == "kingsshield";
            if shield || spec.category != crate::moves::Category::Status {
                self.ability_form(
                    who,
                    if shield {
                        "aegislash-shield"
                    } else {
                        "aegislash-blade"
                    },
                    "aegislash-shield",
                    events,
                );
            }
        }
    }

    pub(crate) fn residual_ability_forms(&mut self, events: &mut Vec<BattleEvent>) {
        for who in self.actors() {
            self.health_ability_form(who, false, events);
            if self.has_ability(who, "hungerswitch") && self.species_is(who, 877, "morpeko") {
                let form = if self.get(who).form.as_deref() == Some("morpeko-hangry") {
                    "morpeko-full-belly"
                } else {
                    "morpeko-hangry"
                };
                self.ability_form(who, form, "morpeko-full-belly", events);
            }
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
        if self.has_ability(who, "shieldsdown") && self.species_is(who, 774, "minior") {
            let core = self
                .get(who)
                .battle_base_form
                .as_ref()
                .and_then(|(form, _)| form.as_deref())
                .unwrap_or("minior-red")
                .trim_end_matches("-meteor")
                .to_owned();
            let meteor = format!("{core}-meteor");
            let form = if self.get(who).hp > self.get(who).max_hp / 2 {
                meteor.as_str()
            } else {
                core.as_str()
            };
            self.ability_form(who, form, &core, events);
        }
        if self.has_ability(who, "schooling")
            && self.species_is(who, 746, "wishiwashi")
            && self.get(who).level >= 20
        {
            let form = if self.get(who).hp > self.get(who).max_hp / 4 {
                "wishiwashi-school"
            } else {
                "wishiwashi-solo"
            };
            self.ability_form(who, form, "wishiwashi-solo", events);
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
        self.update_mimicry(who, events);
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
