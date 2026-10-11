//! Ability forms use canonical form slugs and stats supplied by the host in `Pokemon::forms`.
use crate::engine::Battle;
use crate::items::item_key;
use crate::model::{BattleEvent, ParticipantId, WeatherKind};

impl Battle {
    pub(crate) fn item_form_type(&self, who: ParticipantId) -> Option<crate::model::PokemonType> {
        use crate::model::PokemonType::*;
        if self.get(who).transformed() {
            return Option::None;
        }
        if self.has_ability(who, "multitype") && self.species_is(who, 493, "arceus") {
            return Some(
                match item_key(self.get(who).item.as_deref().unwrap_or_default()).as_str() {
                    "flameplate" => Fire,
                    "splashplate" => Water,
                    "meadowplate" => Grass,
                    "zapplate" => Electric,
                    "icicleplate" => Ice,
                    "fistplate" => Fighting,
                    "toxicplate" => Poison,
                    "earthplate" => Ground,
                    "skyplate" => Flying,
                    "mindplate" => Psychic,
                    "insectplate" => Bug,
                    "stoneplate" => Rock,
                    "spookyplate" => Ghost,
                    "dracoplate" => Dragon,
                    "dreadplate" => Dark,
                    "ironplate" => Steel,
                    "pixieplate" => Fairy,
                    _ => Normal,
                },
            );
        }
        if self.has_ability(who, "rkssystem") && self.species_is(who, 773, "silvally") {
            return Some(
                match item_key(self.get(who).item.as_deref().unwrap_or_default()).as_str() {
                    "firememory" => Fire,
                    "watermemory" => Water,
                    "grassmemory" => Grass,
                    "electricmemory" => Electric,
                    "icememory" => Ice,
                    "fightingmemory" => Fighting,
                    "poisonmemory" => Poison,
                    "groundmemory" => Ground,
                    "flyingmemory" => Flying,
                    "psychicmemory" => Psychic,
                    "bugmemory" => Bug,
                    "rockmemory" => Rock,
                    "ghostmemory" => Ghost,
                    "dragonmemory" => Dragon,
                    "darkmemory" => Dark,
                    "steelmemory" => Steel,
                    "fairymemory" => Fairy,
                    _ => Normal,
                },
            );
        }
        Option::None
    }

    pub(crate) fn form_shields_hit(
        &self,
        who: ParticipantId,
        category: crate::moves::Category,
    ) -> bool {
        if self.get(who).transformed() {
            return false;
        }
        (category != crate::moves::Category::Status
            && self.has_ability(who, "disguise")
            && self.species_is(who, 778, "mimikyu")
            && !self
                .get(who)
                .form
                .as_deref()
                .is_some_and(|id| id.contains("busted")))
            || (category == crate::moves::Category::Physical
                && self.has_ability(who, "iceface")
                && self.species_is(who, 875, "eiscue")
                && self.get(who).form.as_deref() != Some("eiscue-noice"))
    }

    pub(crate) fn restore_ice_face(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        if self.has_ability(who, "iceface")
            && self.species_is(who, 875, "eiscue")
            && !self.get(who).transformed()
            && self
                .weather
                .as_ref()
                .is_some_and(|w| w.kind == WeatherKind::Hail)
            && self.get(who).form.as_deref() == Some("eiscue-noice")
        {
            self.persistent_ability_form(who, "eiscue-ice", events);
        }
    }

    pub(crate) fn persistent_ability_form(
        &mut self,
        who: ParticipantId,
        form: &str,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if !self.change_form(who, form, events) {
            return false;
        }
        let p = self.get_mut(who);
        p.original_form = None;
        p.battle_base_form = Some((p.form.clone(), crate::model::PokemonForm::of(p)));
        true
    }

    pub(crate) fn break_form_shield(
        &mut self,
        who: ParticipantId,
        category: crate::moves::Category,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if !self.form_shields_hit(who, category) {
            return false;
        }
        if self.has_ability(who, "iceface") {
            if let Some((_, base)) = self.get(who).battle_base_form.clone() {
                self.get_mut(who)
                    .forms
                    .entry("eiscue-ice".into())
                    .or_insert(base);
            }
            return self.persistent_ability_form(who, "eiscue-noice", events);
        }
        let form = if self
            .get(who)
            .form
            .as_deref()
            .is_some_and(|id| id.contains("totem"))
        {
            "mimikyu-totem-busted"
        } else {
            "mimikyu-busted"
        };
        if !self.persistent_ability_form(who, form, events) {
            return false;
        }
        self.damage(who, (self.get(who).max_hp / 8).max(1), events);
        true
    }

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

    pub(crate) fn battle_bond_knockout(
        &mut self,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) {
        if self.has_ability(who, "battlebond")
            && self.species_is(who, 658, "greninja")
            && !self.get(who).transformed()
            && self.get(who).form.as_deref() != Some("greninja-ash")
            && self
                .participants(who.side.opposite())
                .iter()
                .any(|p| p.hp > 0)
        {
            self.persistent_ability_form(who, "greninja-ash", events);
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
        if let Some(kind) = self.item_form_type(who) {
            self.get_mut(who).types = vec![kind];
        }
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
