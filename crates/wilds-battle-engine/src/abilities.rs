//! Ability activation and suppression gates shared by engine mechanics.
use crate::engine::Battle;
use crate::items::item_key;
use crate::model::{BattleEvent, ParticipantId};

/// Pokemon Showdown 942cda71, data/abilities.ts: cantsuppress metadata.
fn cannot_suppress(ability: &str) -> bool {
    matches!(
        ability,
        "asoneglastrier"
            | "asonespectrier"
            | "battlebond"
            | "comatose"
            | "disguise"
            | "gulpmissile"
            | "iceface"
            | "multitype"
            | "powerconstruct"
            | "rkssystem"
            | "schooling"
            | "shieldsdown"
            | "stancechange"
            | "terashift"
            | "zenmode"
            | "zerotohero"
    )
}

impl Battle {
    fn ability_terrain_entry(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        use crate::model::{Condition, PokemonType, Rule, Scope, Status};
        let (name, rules) = if self.has_ability(who, "electricsurge") {
            (
                "electric_terrain",
                vec![
                    Rule::BasePower {
                        move_type: Some(PokemonType::Electric),
                        factor: 5325.0 / 4096.0,
                        grounded_only: true,
                    },
                    Rule::BlockGroundedStatus {
                        statuses: vec![Status::Asleep],
                        confusion: false,
                    },
                ],
            )
        } else if self.has_ability(who, "psychicsurge") {
            (
                "psychic_terrain",
                vec![
                    Rule::BasePower {
                        move_type: Some(PokemonType::Psychic),
                        factor: 5325.0 / 4096.0,
                        grounded_only: true,
                    },
                    Rule::BlockPriority {
                        grounded_only: true,
                    },
                ],
            )
        } else if self.has_ability(who, "mistysurge") {
            (
                "misty_terrain",
                vec![
                    Rule::IncomingBasePower {
                        moves: vec![],
                        move_type: Some(PokemonType::Dragon),
                        factor: 0.5,
                        grounded_only: true,
                    },
                    Rule::BlockGroundedStatus {
                        statuses: vec![],
                        confusion: true,
                    },
                ],
            )
        } else if self.has_ability(who, "grassysurge") {
            (
                "grassy_terrain",
                vec![
                    Rule::BasePower {
                        move_type: Some(PokemonType::Grass),
                        factor: 5325.0 / 4096.0,
                        grounded_only: true,
                    },
                    Rule::IncomingBasePower {
                        moves: vec!["earthquake".into(), "bulldoze".into(), "magnitude".into()],
                        move_type: None,
                        factor: 0.5,
                        grounded_only: true,
                    },
                    Rule::HealEachTurn {
                        fraction: 1.0 / 16.0,
                        message: None,
                        grounded_only: true,
                    },
                ],
            )
        } else {
            return;
        };
        if self.start_condition(
            Scope::Field,
            Condition {
                name: name.into(),
                value: 1,
                turns_left: Some(5),
                rules,
                group: Some("terrain".into()),
                end_message: None,
            },
            false,
        ) {
            events.push(BattleEvent::Message(format!(
                "{}'s ability set {}!",
                self.get(who).name,
                name.replace('_', " ")
            )));
        }
    }

    pub(crate) fn drain_health(
        &mut self,
        who: ParticipantId,
        amount: u16,
        ooze: bool,
        events: &mut Vec<BattleEvent>,
    ) {
        if self.get(who).hp == 0 {
            return;
        }
        if ooze {
            self.indirect_damage(who, amount, events);
        } else if self.get(who).hp < self.get(who).max_hp {
            self.heal(who, amount, events);
        }
    }

    pub(crate) fn roll_hit_count(&mut self, actor: ParticipantId, min: u8, max: u8) -> u8 {
        if self.has_ability(actor, "skilllink") {
            return max;
        }
        if (min, max) == (2, 5) {
            match self.rng.range(1, 100) {
                1..=35 => 2,
                36..=70 => 3,
                71..=85 => 4,
                _ => 5,
            }
        } else {
            self.rng.range(min, max)
        }
    }

    pub(crate) fn damp_active(&self) -> bool {
        !self.ability_bypass
            && self
                .actors()
                .into_iter()
                .any(|who| self.has_ability(who, "damp"))
    }

    pub(crate) fn ability_damaging_hit(
        &mut self,
        target: ParticipantId,
        ability: &str,
        hit: &crate::model::HitInfo,
        events: &mut Vec<BattleEvent>,
    ) {
        if hit.damage == 0 || hit.attacker == target {
            return;
        }
        if ability == "perishbody"
            && hit.contact
            && !self
                .get(hit.attacker)
                .conditions
                .iter()
                .any(|c| c.name == "perish_song")
        {
            for who in [hit.attacker, target] {
                if self.get(who).hp > 0 {
                    self.start_condition(
                        crate::model::Scope::Pokemon(who),
                        crate::model::Condition {
                            name: "perish_song".into(),
                            value: 1,
                            turns_left: Some(4),
                            rules: vec![crate::model::Rule::FaintOnExpiry],
                            group: None,
                            end_message: None,
                        },
                        false,
                    );
                }
            }
        }
        if ability == "wanderingspirit" && hit.contact {
            self.swap_abilities(hit.attacker, target, events);
        }
        if ability == "mummy" && hit.contact && !self.has_ability(hit.attacker, "mummy") {
            self.replace_ability(
                hit.attacker,
                crate::model::AbilityState {
                    id: Some("mummy".into()),
                    rules: vec![],
                    bypassable: None,
                },
                events,
            );
        }
        if ability == "innardsout" && self.get(target).hp == 0 {
            self.indirect_damage(hit.attacker, hit.damage, events);
        }
        if ability == "aftermath" && self.get(target).hp == 0 && hit.contact && !self.damp_active()
        {
            self.indirect_damage(
                hit.attacker,
                (self.get(hit.attacker).max_hp / 4).max(1),
                events,
            );
        }
        let last_used = self
            .get(hit.attacker)
            .last_used
            .clone()
            .unwrap_or_else(|| hit.move_id.clone());
        if ability == "cursedbody"
            && hit.move_id != "struggle"
            && !self
                .catalog
                .get(&hit.move_id)
                .is_some_and(|m| m.has_flag("future_move"))
            && self
                .get(hit.attacker)
                .move_pp
                .get(&last_used)
                .is_some_and(|pp| *pp > 0)
            && self.rng.chance(0.3)
        {
            self.start_condition(
                crate::model::Scope::Pokemon(hit.attacker),
                crate::model::Condition {
                    name: "disable".into(),
                    value: 0,
                    turns_left: Some(4),
                    rules: vec![crate::model::Rule::BlockMove {
                        id: last_used,
                        message: Some("{name}'s move is disabled!".into()),
                    }],
                    group: None,
                    end_message: None,
                },
                false,
            );
        }
        if ability == "sandspit" {
            self.start_weather(crate::model::WeatherKind::Sandstorm, 5, events);
        }
        if ability == "cottondown" {
            for who in self.actors() {
                if who != target {
                    self.change_stat_from(target, who, crate::model::Stat::Speed, -1, events);
                }
            }
        }
        if self.get(target).hp > 0 && ability == "stamina" {
            self.change_stat(target, crate::model::Stat::Defense, 1, events);
        }
        let typed_boost = match (ability, hit.move_type) {
            ("steamengine", crate::model::PokemonType::Water | crate::model::PokemonType::Fire) => {
                Some((crate::model::Stat::Speed, 6))
            }
            ("watercompaction", crate::model::PokemonType::Water) => {
                Some((crate::model::Stat::Defense, 2))
            }
            ("justified", crate::model::PokemonType::Dark) => Some((crate::model::Stat::Attack, 1)),
            _ => None,
        };
        if self.get(target).hp > 0
            && let Some((stat, stages)) = typed_boost
        {
            self.change_stat(target, stat, stages, events);
        }
        if self.get(target).hp > 0
            && ability == "weakarmor"
            && hit.category == crate::moves::Category::Physical
        {
            self.change_stat(target, crate::model::Stat::Defense, -1, events);
            self.change_stat(target, crate::model::Stat::Speed, 2, events);
        }
        if matches!(ability, "gooey" | "tanglinghair")
            && hit.contact
            && self.get(hit.attacker).hp > 0
        {
            self.change_stat_from(target, hit.attacker, crate::model::Stat::Speed, -1, events);
        }
        if ability == "cutecharm" && hit.contact && self.rng.chance(0.3) {
            self.start_condition(
                crate::model::Scope::Pokemon(hit.attacker),
                crate::model::Condition {
                    name: "attract".into(),
                    value: 0,
                    turns_left: None,
                    rules: vec![crate::model::Rule::InfatuatedBy(target)],
                    group: None,
                    end_message: None,
                },
                false,
            );
        }
        if matches!(ability, "roughskin" | "ironbarbs") && hit.contact {
            self.indirect_damage(
                hit.attacker,
                (self.get(hit.attacker).max_hp / 8).max(1),
                events,
            );
        }
        let contact_status = match ability {
            "static" => Some(crate::model::AppliedStatus::Paralyzed),
            "flamebody" => Some(crate::model::AppliedStatus::Burned),
            "poisonpoint" => Some(crate::model::AppliedStatus::Poisoned),
            _ => None,
        };
        if let Some(status) = contact_status
            && hit.contact
            && self.rng.chance(0.3)
        {
            self.inflict_status(
                target,
                hit.attacker,
                status,
                false,
                crate::model::PokemonType::None,
                false,
                events,
            );
        }
        if ability == "effectspore"
            && hit.contact
            && !self
                .types_of(hit.attacker)
                .contains(&crate::model::PokemonType::Grass)
            && !self.has_ability(hit.attacker, "overcoat")
        {
            let roll = self.rng.fraction();
            let status = if roll < 0.11 {
                Some(crate::model::AppliedStatus::Asleep)
            } else if roll < 0.21 {
                Some(crate::model::AppliedStatus::Paralyzed)
            } else if roll < 0.3 {
                Some(crate::model::AppliedStatus::Poisoned)
            } else {
                None
            };
            if let Some(status) = status {
                self.inflict_status(
                    target,
                    hit.attacker,
                    status,
                    false,
                    crate::model::PokemonType::None,
                    false,
                    events,
                );
            }
        }
    }

    pub(crate) fn ability_attacking_hit(
        &mut self,
        target: ParticipantId,
        hit: &crate::model::HitInfo,
        events: &mut Vec<BattleEvent>,
    ) {
        if hit.damage > 0
            && hit.contact
            && self.has_ability(hit.attacker, "poisontouch")
            && !self.has_ability(target, "shielddust")
            && self.rng.chance(0.3)
        {
            self.inflict_status(
                hit.attacker,
                target,
                crate::model::AppliedStatus::Poisoned,
                false,
                crate::model::PokemonType::None,
                false,
                events,
            );
        }
    }

    /// Weather keeps its timer while its battle effects are suppressed.
    pub(crate) fn effective_weather(&self) -> Option<crate::model::WeatherKind> {
        if self
            .actors()
            .into_iter()
            .any(|who| self.has_ability(who, "cloudnine") || self.has_ability(who, "airlock"))
        {
            None
        } else {
            self.weather.as_ref().map(|weather| weather.kind)
        }
    }

    pub(crate) fn ability_state(&self, who: ParticipantId) -> crate::model::AbilityState {
        let p = self.get(who);
        crate::model::AbilityState {
            id: p.ability.clone(),
            rules: p.ability_rules.clone(),
            bypassable: p.ability_bypassable,
        }
    }

    pub(crate) fn replace_ability(
        &mut self,
        who: ParticipantId,
        state: crate::model::AbilityState,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if self.get(who).hp == 0
            || !self.is_active(who)
            || self
                .get(who)
                .ability
                .as_deref()
                .is_some_and(|id| cannot_suppress(&item_key(id)))
            || state
                .id
                .as_deref()
                .is_none_or(|id| id.is_empty() || cannot_suppress(&item_key(id)))
        {
            return false;
        }
        self.install_ability(who, state, events);
        self.ability_on_entry(who, events);
        true
    }

    pub(crate) fn install_ability(
        &mut self,
        who: ParticipantId,
        state: crate::model::AbilityState,
        events: &mut Vec<BattleEvent>,
    ) {
        let original = self.ability_state(who);
        let pokemon = self.get_mut(who);
        pokemon.original_ability.get_or_insert(original);
        pokemon.ability = state.id;
        pokemon.ability_rules = state.rules;
        pokemon.ability_bypassable = state.bypassable;
        pokemon.conditions.retain(|c| c.name != "flash_fire");
        events.push(BattleEvent::Message(format!(
            "{} acquired {}!",
            pokemon.name,
            pokemon.ability.as_deref().unwrap_or_default()
        )));
    }

    pub(crate) fn swap_abilities(
        &mut self,
        a: ParticipantId,
        b: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if a == b
            || [a, b].iter().any(|id| {
                self.get(*id).hp == 0
                    || !self.is_active(*id)
                    || self.get(*id).ability.as_deref().is_none_or(|ability| {
                        ability.is_empty() || ability_flag(&item_key(ability), "failskillswap")
                    })
            })
        {
            return false;
        }
        let first = self.ability_state(a);
        let second = self.ability_state(b);
        self.install_ability(a, second, events);
        self.install_ability(b, first, events);
        // Both abilities change before either activation; the recipient starts first.
        self.ability_on_entry(b, events);
        self.ability_on_entry(a, events);
        true
    }

    pub(crate) fn ability_stage_delta(&self, who: ParticipantId, delta: i8) -> i8 {
        if self.has_ability(who, "contrary") {
            delta.saturating_neg()
        } else if self.has_ability(who, "simple") {
            delta.saturating_mul(2)
        } else {
            delta
        }
    }

    pub(crate) fn finish_ability_knockouts(&mut self, events: &mut Vec<BattleEvent>) {
        // All targets and recoil resolve before knockout rewards change attacking stats.
        for actor in std::mem::take(&mut self.pending_knockouts) {
            if self.has_ability(actor, "beastboost") {
                use crate::model::{CombatStat, Stat};
                let stats = [
                    (CombatStat::Attack, Stat::Attack),
                    (CombatStat::Defense, Stat::Defense),
                    (CombatStat::SpAttack, Stat::SpAttack),
                    (CombatStat::SpDefense, Stat::SpDefense),
                    (CombatStat::Speed, Stat::Speed),
                ];
                let (_, (_, stat)) = stats
                    .iter()
                    .enumerate()
                    .max_by_key(|(index, (raw, _))| {
                        (self.get(actor).raw_stat(*raw), std::cmp::Reverse(*index))
                    })
                    .unwrap();
                self.change_stat(actor, *stat, 1, events);
            }
            if self.has_ability(actor, "asoneglastrier")
                || self.has_ability(actor, "moxie")
                || self.has_ability(actor, "chillingneigh")
            {
                self.change_stat(actor, crate::model::Stat::Attack, 1, events);
            }
            if self.has_ability(actor, "asonespectrier") || self.has_ability(actor, "grimneigh") {
                self.change_stat(actor, crate::model::Stat::SpAttack, 1, events);
            }
        }
    }

    pub(crate) fn ability_weather_residual(
        &mut self,
        who: ParticipantId,
        kind: crate::model::WeatherKind,
        events: &mut Vec<BattleEvent>,
    ) {
        use crate::model::WeatherKind::*;
        if kind == Sun && self.has_ability(who, "solarpower") {
            self.indirect_damage(who, (self.get(who).max_hp / 8).max(1), events);
        }
        if self.has_ability(who, "dryskin") {
            match kind {
                Rain => {
                    self.heal(who, (self.get(who).max_hp / 8).max(1), events);
                }
                Sun => {
                    self.indirect_damage(who, (self.get(who).max_hp / 8).max(1), events);
                }
                _ => {}
            }
        }
        if kind == Hail && self.has_ability(who, "icebody") {
            self.heal(who, (self.get(who).max_hp / 16).max(1), events);
        }
        if kind == Rain && self.has_ability(who, "raindish") {
            self.heal(who, (self.get(who).max_hp / 16).max(1), events);
        }
    }

    pub(crate) fn ability_weather_immunity(
        &self,
        who: ParticipantId,
        kind: crate::model::WeatherKind,
    ) -> bool {
        (kind == crate::model::WeatherKind::Sandstorm
            && (self.has_ability(who, "sandrush")
                || self.has_ability(who, "sandveil")
                || self.has_ability(who, "sandforce")))
            || self.has_ability(who, "overcoat")
            || (kind == crate::model::WeatherKind::Hail
                && (self.has_ability(who, "icebody") || self.has_ability(who, "snowcloak")))
    }

    pub(crate) fn ability_residual(&mut self, events: &mut Vec<BattleEvent>) {
        let mut order = self.actors();
        order.sort_by_key(|&who| std::cmp::Reverse(self.action_speed(who)));
        for who in order {
            if self.has_ability(who, "speedboost") && self.get(who).entered_turn < self.turn {
                self.change_stat(who, crate::model::Stat::Speed, 1, events);
            }
        }
    }

    pub(crate) fn ability_on_exit(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        // Run before volatile state and suppression are cleared, and only while alive.
        if self.has_ability(who, "regenerator") {
            self.heal_unchecked(who, self.get(who).max_hp / 3, events);
        }
        if self.has_ability(who, "naturalcure") {
            self.cure_status(who, None, events);
        }
    }

    pub(crate) fn ability_on_entry(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        self.ability_terrain_entry(who, events);
        if self.has_ability(who, "screencleaner") {
            let mut cleared = false;
            for side in [crate::model::Side::Allies, crate::model::Side::Foes] {
                for screen in ["reflect", "light_screen", "aurora_veil"] {
                    cleared |= self.end_condition(crate::model::Scope::Side(side), screen);
                }
            }
            if cleared {
                events.push(BattleEvent::Message(
                    "Screen Cleaner removed the screens!".into(),
                ));
            }
        }
        if self.has_ability(who, "curiousmedicine") {
            for ally in self.actors() {
                if ally != who && ally.side == who.side && !self.get(ally).stages.is_empty() {
                    self.get_mut(ally).stages.clear();
                    events.push(BattleEvent::Message(format!(
                        "{}'s stat changes were removed by Curious Medicine!",
                        self.get(ally).name
                    )));
                }
            }
        }
        if self.has_ability(who, "download") {
            let (defense, special) = self
                .actors()
                .into_iter()
                .filter(|foe| foe.side != who.side)
                .fold((0.0, 0.0), |(a, b), foe| {
                    (
                        a + self.battle_staged_stat(foe, crate::model::CombatStat::Defense),
                        b + self.battle_staged_stat(foe, crate::model::CombatStat::SpDefense),
                    )
                });
            if defense > 0.0 && defense >= special {
                self.change_stat(who, crate::model::Stat::SpAttack, 1, events);
            } else if special > 0.0 {
                self.change_stat(who, crate::model::Stat::Attack, 1, events);
            }
        }
        if self.has_ability(who, "dauntlessshield") {
            self.change_stat(who, crate::model::Stat::Defense, 1, events);
        }
        // Sword/Shield activates this on every entry, unlike the once-per-battle Gen 9 rule.
        if self.has_ability(who, "intrepidsword") {
            self.change_stat(who, crate::model::Stat::Attack, 1, events);
        }
        if self.has_ability(who, "snowwarning") {
            self.start_weather(crate::model::WeatherKind::Hail, 5, events);
        }
        if self.has_ability(who, "sandstream") {
            self.start_weather(crate::model::WeatherKind::Sandstorm, 5, events);
        }
        if self.has_ability(who, "drought") {
            self.start_weather(crate::model::WeatherKind::Sun, 5, events);
        }
        if self.has_ability(who, "drizzle") {
            self.start_weather(crate::model::WeatherKind::Rain, 5, events);
        }
        self.pastel_veil_entry(events);
        if self.has_ability(who, "owntempo") {
            self.get_mut(who).confused_turns = None;
        }
        if self.has_ability(who, "insomnia") || self.has_ability(who, "vitalspirit") {
            self.cure_status(who, Some(crate::model::Status::Asleep), events);
        }
        if !self.has_ability(who, "intimidate") {
            return;
        }
        events.push(BattleEvent::Message(format!(
            "{}'s Intimidate activated!",
            self.get(who).name
        )));
        for target in self.actors() {
            if target.side == who.side
                || !self.adjacent(who, target)
                || self.get(target).substitute_hp > 0
            {
                continue;
            }
            let amount = if self.has_ability(target, "guarddog") {
                1
            } else {
                -1
            };
            let immune = (self.ability_stage_delta(target, amount) < 0
                && self.blocks_stat_drops_from(who, target))
                || [
                    "hypercutter",
                    "innerfocus",
                    "oblivious",
                    "owntempo",
                    "scrappy",
                ]
                .iter()
                .any(|ability| self.has_ability(target, ability));
            if immune {
                events.push(BattleEvent::Message(format!(
                    "{} resisted Intimidate!",
                    self.get(target).name
                )));
                continue;
            }
            let changed =
                self.change_stat_from(who, target, crate::model::Stat::Attack, amount, events);
            if changed != 0 && self.has_ability(target, "rattled") {
                self.change_stat(target, crate::model::Stat::Speed, 1, events);
            }
        }
    }

    pub(crate) fn has_ability(&self, who: ParticipantId, ability: &str) -> bool {
        self.ability_active(who)
            && self
                .get(who)
                .ability
                .as_deref()
                .is_some_and(|id| item_key(id) == ability)
    }

    pub(crate) fn suppress_ability(
        &mut self,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let pokemon = self.get(who);
        if pokemon.hp == 0
            || pokemon.ability_suppressed
            || pokemon
                .ability
                .as_deref()
                .is_none_or(|id| id.is_empty() || cannot_suppress(&item_key(id)))
        {
            return false;
        }
        let pokemon = self.get_mut(who);
        pokemon.ability_suppressed = true;
        events.push(BattleEvent::Message(format!(
            "{}'s ability was suppressed!",
            pokemon.name
        )));
        true
    }
}

/// Breakable metadata is distinct from suppression protection: Disguise is an example
/// of an ability which cannot be suppressed but can be bypassed by a move.
fn bypassable(ability: &str) -> bool {
    matches!(
        ability,
        "armortail"
            | "aromaveil"
            | "aurabreak"
            | "battlearmor"
            | "bigpecks"
            | "bulletproof"
            | "clearbody"
            | "contrary"
            | "damp"
            | "dazzling"
            | "disguise"
            | "dryskin"
            | "eartheater"
            | "filter"
            | "flashfire"
            | "flowergift"
            | "flowerveil"
            | "fluffy"
            | "friendguard"
            | "furcoat"
            | "goodasgold"
            | "grasspelt"
            | "guarddog"
            | "heatproof"
            | "heavymetal"
            | "hypercutter"
            | "iceface"
            | "icescales"
            | "illuminate"
            | "immunity"
            | "innerfocus"
            | "insomnia"
            | "keeneye"
            | "leafguard"
            | "levitate"
            | "lightmetal"
            | "lightningrod"
            | "limber"
            | "magicbounce"
            | "magmaarmor"
            | "marvelscale"
            | "mindseye"
            | "mirrorarmor"
            | "motordrive"
            | "multiscale"
            | "oblivious"
            | "overcoat"
            | "owntempo"
            | "pastelveil"
            | "punkrock"
            | "purifyingsalt"
            | "queenlymajesty"
            | "sandveil"
            | "sapsipper"
            | "shellarmor"
            | "shielddust"
            | "simple"
            | "snowcloak"
            | "solidrock"
            | "soundproof"
            | "stickyhold"
            | "stormdrain"
            | "sturdy"
            | "suctioncups"
            | "sweetveil"
            | "tangledfeet"
            | "telepathy"
            | "terashell"
            | "thermalexchange"
            | "thickfat"
            | "unaware"
            | "vitalspirit"
            | "voltabsorb"
            | "waterabsorb"
            | "waterbubble"
            | "waterveil"
            | "wellbakedbody"
            | "whitesmoke"
            | "windrider"
            | "wonderguard"
            | "wonderskin"
    )
}

impl Battle {
    pub(crate) fn ability_active(&self, who: ParticipantId) -> bool {
        let pokemon = self.get(who);
        pokemon.hp > 0
            && self.is_active(who)
            && !pokemon.ability_suppressed
            && pokemon.ability.as_deref().is_some_and(|id| {
                if id.is_empty() {
                    return false;
                }
                let ignored = self.ignored_ability == Some(who)
                    && pokemon
                        .ability_bypassable
                        .unwrap_or_else(|| bypassable(&item_key(id)));
                !ignored
            })
    }

    pub(crate) fn ability_traps(&self, who: ParticipantId) -> bool {
        self.actors().into_iter().any(|source| {
            source.side != who.side
                && self.adjacent(source, who)
                && ((self.has_ability(source, "shadowtag") && !self.has_ability(who, "shadowtag"))
                    || (self.has_ability(source, "arenatrap") && self.on_ground(who))
                    || (self.has_ability(source, "magnetpull")
                        && self
                            .types_of(who)
                            .contains(&crate::model::PokemonType::Steel)))
        })
    }

    pub(crate) fn effective_weight(&self, who: ParticipantId) -> f32 {
        let weight = self.get(who).weight;
        if self.has_ability(who, "heavymetal") {
            (weight * 2.0).max(0.1)
        } else if self.has_ability(who, "lightmetal") {
            ((weight * 5.0).floor() / 10.0).max(0.1)
        } else {
            weight.max(0.1)
        }
    }

    pub(crate) fn ability_stat_factor(
        &self,
        who: ParticipantId,
        stat: crate::model::CombatStat,
    ) -> f32 {
        use crate::model::CombatStat::*;
        if stat == Defense
            && self.has_ability(who, "grasspelt")
            && self
                .field_conditions
                .iter()
                .any(|c| c.name == "grassy_terrain")
        {
            return 1.5;
        }
        if stat == Speed
            && self.has_ability(who, "surgesurfer")
            && self
                .field_conditions
                .iter()
                .any(|c| c.name == "electric_terrain")
        {
            return 2.0;
        }
        if stat == SpAttack
            && self.has_ability(who, "solarpower")
            && self.effective_weather() == Some(crate::model::WeatherKind::Sun)
        {
            return 1.5;
        }
        if stat == Speed
            && self.has_ability(who, "sandrush")
            && self.effective_weather() == Some(crate::model::WeatherKind::Sandstorm)
        {
            return 2.0;
        }
        if stat == Speed
            && self.has_ability(who, "slushrush")
            && self.effective_weather() == Some(crate::model::WeatherKind::Hail)
        {
            return 2.0;
        }
        if stat == Speed
            && self.has_ability(who, "chlorophyll")
            && self.effective_weather() == Some(crate::model::WeatherKind::Sun)
        {
            return 2.0;
        }
        if stat == Speed
            && self.has_ability(who, "swiftswim")
            && self.effective_weather() == Some(crate::model::WeatherKind::Rain)
        {
            return 2.0;
        }
        if stat == Attack && self.has_ability(who, "hustle") {
            return 1.5;
        }
        if matches!(stat, Attack | SpAttack)
            && self.has_ability(who, "defeatist")
            && self.get(who).hp <= self.get(who).max_hp / 2
        {
            return 0.5;
        }
        if stat == SpAttack
            && (self.has_ability(who, "plus") || self.has_ability(who, "minus"))
            && self.actors().into_iter().any(|ally| {
                ally != who
                    && ally.side == who.side
                    && (self.has_ability(ally, "plus") || self.has_ability(ally, "minus"))
            })
        {
            return 1.5;
        }
        if (stat == Defense && self.has_ability(who, "furcoat"))
            || (stat == Attack
                && (self.has_ability(who, "hugepower") || self.has_ability(who, "purepower")))
        {
            2.0
        } else if self.get(who).status.is_some()
            && ((stat == Defense && self.has_ability(who, "marvelscale"))
                || (stat == Attack && self.has_ability(who, "guts"))
                || (stat == Speed && self.has_ability(who, "quickfeet")))
        {
            1.5
        } else {
            1.0
        }
    }

    pub(crate) fn ability_accuracy_factor(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        category: crate::moves::Category,
    ) -> f32 {
        let mut factor = if self.has_ability(actor, "compoundeyes") {
            1.3
        } else if category == crate::moves::Category::Physical && self.has_ability(actor, "hustle")
        {
            0.8
        } else {
            1.0
        };
        for ally in self.actors() {
            if ally.side == actor.side && self.has_ability(ally, "victorystar") {
                factor *= 1.1;
            }
        }
        if self.has_ability(target, "sandveil")
            && self.effective_weather() == Some(crate::model::WeatherKind::Sandstorm)
        {
            factor *= 0.8;
        }
        if self.has_ability(target, "snowcloak")
            && self.effective_weather() == Some(crate::model::WeatherKind::Hail)
        {
            factor *= 0.8;
        }
        factor
    }

    pub(crate) fn status_power_factor(
        &self,
        who: ParticipantId,
        category: crate::moves::Category,
    ) -> f32 {
        if (self.get(who).status == Some(crate::model::Status::Burned)
            && category == crate::moves::Category::Special
            && self.has_ability(who, "flareboost"))
            || (matches!(
                self.get(who).status,
                Some(crate::model::Status::Poisoned | crate::model::Status::BadlyPoisoned)
            ) && category == crate::moves::Category::Physical
                && self.has_ability(who, "toxicboost"))
        {
            1.5
        } else {
            1.0
        }
    }

    /// Status moves must use the same type absorption as damaging moves.
    /// Damage-only immunities such as Wonder Guard and Telepathy do not apply here.
    pub(crate) fn ability_blocks_status_move(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &crate::moves::MoveSpec,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if spec.category != crate::moves::Category::Status || actor == target {
            return false;
        }
        if self.ability_bypass {
            self.ignored_ability = Some(target);
        }
        if spec.id == "captivate" && self.has_ability(target, "oblivious") {
            self.immune_message(target, events);
            return true;
        }
        let kind = self.converted_type(actor, spec.move_type);
        let absorbs = (kind == crate::model::PokemonType::Electric
            && (self.has_ability(target, "voltabsorb")
                || self.has_ability(target, "motordrive")
                || self.has_ability(target, "lightningrod")))
            || (kind == crate::model::PokemonType::Water
                && (self.has_ability(target, "waterabsorb")
                    || self.has_ability(target, "stormdrain")
                    || self.has_ability(target, "dryskin")))
            || (kind == crate::model::PokemonType::Fire && self.has_ability(target, "flashfire"))
            || (kind == crate::model::PokemonType::Grass && self.has_ability(target, "sapsipper"));
        if absorbs
            && self.ability_blocks_damage(actor, target, kind, 2.0, false, false, true, events)
        {
            return true;
        }
        // Thunder Wave uniquely checks type effectiveness even as a status move.
        if spec.id == "thunder-wave" && self.move_effectiveness(actor, target, kind) == 0.0 {
            self.immune_message(target, events);
            return true;
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ability_blocks_damage(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        kind: crate::model::PokemonType,
        effectiveness: f32,
        sound: bool,
        bullet: bool,
        hits_airborne: bool,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        use crate::model::PokemonType::{Electric, Fire, Grass, Ground, Water};
        if actor == target {
            return false;
        }
        let heal = (kind == Water
            && (self.has_ability(target, "waterabsorb") || self.has_ability(target, "dryskin")))
            || (kind == Electric && self.has_ability(target, "voltabsorb"));
        let boost = if kind == Electric && self.has_ability(target, "motordrive") {
            Some(crate::model::Stat::Speed)
        } else if (kind == Electric && self.has_ability(target, "lightningrod"))
            || (kind == Water && self.has_ability(target, "stormdrain"))
        {
            Some(crate::model::Stat::SpAttack)
        } else if kind == Grass && self.has_ability(target, "sapsipper") {
            Some(crate::model::Stat::Attack)
        } else {
            None
        };
        let flash = kind == Fire && self.has_ability(target, "flashfire");
        let blocked = heal
            || boost.is_some()
            || flash
            || (kind == Ground
                && !hits_airborne
                && !self.is_grounded(target)
                && self.has_ability(target, "levitate"))
            || (effectiveness <= 1.0 && self.has_ability(target, "wonderguard"))
            || (sound && self.has_ability(target, "soundproof"))
            || (bullet && self.has_ability(target, "bulletproof"))
            || (actor.side == target.side && self.has_ability(target, "telepathy"));
        if !blocked {
            return false;
        }
        events.push(BattleEvent::Message(format!(
            "{}'s ability blocked the move!",
            self.get(target).name
        )));
        self.move_failed = true;
        if heal {
            self.heal(target, (self.get(target).max_hp / 4).max(1), events);
        }
        if let Some(stat) = boost {
            self.change_stat(target, stat, 1, events);
        }
        if flash {
            self.set_mark(crate::model::Scope::Pokemon(target), "flash_fire", 1, None);
        }
        true
    }

    pub(crate) fn normalizable_move(move_id: &str) -> bool {
        !matches!(
            crate::items::item_key(move_id).as_str(),
            "hiddenpower"
                | "judgment"
                | "multiattack"
                | "naturalgift"
                | "revelationdance"
                | "struggle"
                | "technoblast"
                | "terrainpulse"
                | "weatherball"
        )
    }

    pub(crate) fn normal_ability_type(
        &self,
        actor: ParticipantId,
    ) -> Option<crate::model::PokemonType> {
        self.has_ability(actor, "refrigerate")
            .then_some(crate::model::PokemonType::Ice)
            .or_else(|| {
                self.has_ability(actor, "aerilate")
                    .then_some(crate::model::PokemonType::Flying)
            })
            .or_else(|| {
                self.has_ability(actor, "pixilate")
                    .then_some(crate::model::PokemonType::Fairy)
            })
            .or_else(|| {
                self.has_ability(actor, "galvanize")
                    .then_some(crate::model::PokemonType::Electric)
            })
    }

    pub(crate) fn team_ability_power_factor(
        &self,
        actor: ParticipantId,
        attack: &crate::engine::Attack,
    ) -> f32 {
        let mut factor = 1.0;
        let aura_factor = if !self.ability_bypass
            && self
                .actors()
                .into_iter()
                .any(|who| self.has_ability(who, "aurabreak"))
        {
            0.75
        } else {
            5448.0 / 4096.0
        };
        if attack.move_type == crate::model::PokemonType::Dark
            && self
                .actors()
                .into_iter()
                .any(|who| self.has_ability(who, "darkaura"))
        {
            factor *= aura_factor;
        }
        if attack.move_type == crate::model::PokemonType::Fairy
            && self
                .actors()
                .into_iter()
                .any(|who| self.has_ability(who, "fairyaura"))
        {
            factor *= aura_factor;
        }
        for ally in self.actors() {
            if ally.side != actor.side {
                continue;
            }
            if ally != actor && self.has_ability(ally, "powerspot") {
                factor *= 1.3;
            }
            if self.has_ability(ally, "steelyspirit")
                && attack.move_type == crate::model::PokemonType::Steel
            {
                factor *= 1.5;
            }
            if ally != actor
                && self.has_ability(ally, "battery")
                && attack.category == crate::moves::Category::Special
            {
                factor *= 1.3;
            }
        }
        factor
    }

    pub(crate) fn ability_base_power_factor(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        attack: &crate::engine::Attack,
    ) -> f32 {
        if self.has_ability(actor, "reckless")
            && self.catalog.get(attack.move_id).is_some_and(|spec| {
                spec.has_flag("crash_damage")
                    || spec.effects.iter().any(|effect| {
                        matches!(
                            effect,
                            crate::moves::Effect::Damage {
                                recoil: Some(_),
                                ..
                            }
                        )
                    })
            })
        {
            return 1.2;
        }
        if self.has_ability(actor, "sandforce")
            && self.effective_weather() == Some(crate::model::WeatherKind::Sandstorm)
            && matches!(
                attack.move_type,
                crate::model::PokemonType::Rock
                    | crate::model::PokemonType::Ground
                    | crate::model::PokemonType::Steel
            )
        {
            return 1.3;
        }
        if self.has_ability(actor, "rivalry") {
            let a = self.get(actor).gender;
            let b = self.get(target).gender;
            if a != crate::model::Gender::Genderless && b != crate::model::Gender::Genderless {
                return if a == b { 1.25 } else { 0.75 };
            }
        }
        if self.has_ability(actor, "steelworker")
            && attack.move_type == crate::model::PokemonType::Steel
        {
            return 1.5;
        }
        if self.has_ability(actor, "transistor")
            && (attack.move_type == crate::model::PokemonType::Electric)
        {
            return 1.5;
        }
        if self.has_ability(actor, "dragonsmaw")
            && (attack.move_type == crate::model::PokemonType::Dragon)
        {
            return 1.5;
        }
        if self.has_ability(actor, "ironfist")
            && (self
                .catalog
                .get(attack.move_id)
                .is_some_and(|m| m.has_flag("punch")))
        {
            return 1.2;
        }
        if self.has_ability(actor, "strongjaw")
            && (self
                .catalog
                .get(attack.move_id)
                .is_some_and(|m| m.has_flag("bite")))
        {
            return 1.5;
        }
        if self.has_ability(actor, "toughclaws") && (attack.contact) {
            return 1.3;
        }
        if self.has_ability(actor, "technician") && (attack.power <= 60) {
            return 1.5;
        }
        if self.has_ability(actor, "overgrow")
            && (attack.move_type == crate::model::PokemonType::Grass
                && u32::from(self.get(actor).hp) * 3 <= u32::from(self.get(actor).max_hp))
        {
            return 1.5;
        }
        if self.has_ability(actor, "blaze")
            && (attack.move_type == crate::model::PokemonType::Fire
                && u32::from(self.get(actor).hp) * 3 <= u32::from(self.get(actor).max_hp))
        {
            return 1.5;
        }
        if self.has_ability(actor, "torrent")
            && (attack.move_type == crate::model::PokemonType::Water
                && u32::from(self.get(actor).hp) * 3 <= u32::from(self.get(actor).max_hp))
        {
            return 1.5;
        }
        if self.has_ability(actor, "swarm")
            && (attack.move_type == crate::model::PokemonType::Bug
                && u32::from(self.get(actor).hp) * 3 <= u32::from(self.get(actor).max_hp))
        {
            return 1.5;
        }
        if self.has_ability(actor, "megalauncher")
            && self
                .catalog
                .get(attack.move_id)
                .is_some_and(|m| m.has_flag("pulse"))
        {
            return 1.5;
        }
        if Self::normalizable_move(attack.move_id)
            && (self.has_ability(actor, "normalize")
                || (self.normal_ability_type(actor).is_some()
                    && self
                        .catalog
                        .get(attack.move_id)
                        .is_some_and(|spec| spec.move_type == crate::model::PokemonType::Normal)))
        {
            1.2
        } else {
            1.0
        }
    }

    pub(crate) fn ability_damage_factor(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        attack: &crate::engine::Attack,
        effectiveness: f32,
    ) -> f32 {
        use crate::model::PokemonType::*;
        let mut factor = 1.0;
        if !self.ability_bypass {
            for ally in self.actors() {
                if ally != target
                    && ally.side == target.side
                    && self.has_ability(ally, "friendguard")
                {
                    factor *= 0.75;
                }
            }
        }
        if self.has_ability(actor, "neuroforce") && effectiveness > 1.0 {
            factor *= 1.25;
        }
        if self.has_ability(actor, "tintedlens") && effectiveness > 0.0 && effectiveness < 1.0 {
            factor *= 2.0;
        }
        if self.get(target).hp == self.get(target).max_hp
            && (self.has_ability(target, "multiscale") || self.has_ability(target, "shadowshield"))
        {
            factor *= 0.5;
        }
        if effectiveness > 1.0
            && (self.has_ability(target, "filter")
                || self.has_ability(target, "solidrock")
                || self.has_ability(target, "prismarmor"))
        {
            factor *= 0.75;
        }
        if attack.category == crate::moves::Category::Special
            && self.has_ability(target, "icescales")
        {
            factor *= 0.5;
        }
        if self.has_ability(target, "fluffy") {
            if attack.contact && !self.has_ability(actor, "longreach") {
                factor *= 0.5;
            }
            if attack.move_type == Fire {
                factor *= 2.0;
            }
        }
        if matches!(attack.move_type, Fire | Ice) && self.has_ability(target, "thickfat") {
            factor *= 0.5;
        }
        if attack.move_type == Fire && self.has_ability(target, "heatproof") {
            factor *= 0.5;
        }
        if attack.move_type == Fire && self.has_ability(target, "waterbubble") {
            factor *= 0.5;
        }
        if attack.move_type == Water && self.has_ability(actor, "waterbubble") {
            factor *= 2.0;
        }
        if attack.sound && self.has_ability(actor, "punkrock") {
            factor *= 1.3;
        }
        if attack.sound && self.has_ability(target, "punkrock") {
            factor *= 0.5;
        }
        if attack.move_type == Fire
            && self.has_ability(actor, "flashfire")
            && self
                .get(actor)
                .conditions
                .iter()
                .any(|condition| condition.name == "flash_fire")
        {
            factor *= 1.5;
        }
        factor
    }
}

pub(crate) fn ability_flag(ability: &str, flag: &str) -> bool {
    match flag {
        "cantsuppress" => cannot_suppress(ability),
        "failroleplay" => matches!(
            ability,
            "asoneglastrier"
                | "asonespectrier"
                | "battlebond"
                | "comatose"
                | "commander"
                | "disguise"
                | "embodyaspectcornerstone"
                | "embodyaspecthearthflame"
                | "embodyaspectteal"
                | "embodyaspectwellspring"
                | "flowergift"
                | "forecast"
                | "hungerswitch"
                | "iceface"
                | "illusion"
                | "imposter"
                | "multitype"
                | "neutralizinggas"
                | "poisonpuppeteer"
                | "powerconstruct"
                | "powerofalchemy"
                | "protosynthesis"
                | "quarkdrive"
                | "receiver"
                | "rkssystem"
                | "schooling"
                | "shieldsdown"
                | "stancechange"
                | "teraformzero"
                | "terashell"
                | "terashift"
                | "trace"
                | "wonderguard"
                | "zenmode"
                | "zerotohero"
        ),
        "failskillswap" => matches!(
            ability,
            "asoneglastrier"
                | "asonespectrier"
                | "battlebond"
                | "comatose"
                | "commander"
                | "disguise"
                | "embodyaspectcornerstone"
                | "embodyaspecthearthflame"
                | "embodyaspectteal"
                | "embodyaspectwellspring"
                | "hungerswitch"
                | "iceface"
                | "illusion"
                | "multitype"
                | "neutralizinggas"
                | "poisonpuppeteer"
                | "powerconstruct"
                | "protosynthesis"
                | "quarkdrive"
                | "rkssystem"
                | "schooling"
                | "shieldsdown"
                | "stancechange"
                | "teraformzero"
                | "terashell"
                | "terashift"
                | "wonderguard"
                | "zenmode"
                | "zerotohero"
        ),
        "noentrain" => matches!(
            ability,
            "asoneglastrier"
                | "asonespectrier"
                | "battlebond"
                | "comatose"
                | "commander"
                | "disguise"
                | "embodyaspectcornerstone"
                | "embodyaspecthearthflame"
                | "embodyaspectteal"
                | "embodyaspectwellspring"
                | "flowergift"
                | "forecast"
                | "hungerswitch"
                | "iceface"
                | "illusion"
                | "imposter"
                | "multitype"
                | "neutralizinggas"
                | "poisonpuppeteer"
                | "powerconstruct"
                | "powerofalchemy"
                | "protosynthesis"
                | "quarkdrive"
                | "receiver"
                | "rkssystem"
                | "schooling"
                | "shieldsdown"
                | "stancechange"
                | "teraformzero"
                | "terashell"
                | "terashift"
                | "trace"
                | "wonderguard"
                | "zenmode"
                | "zerotohero"
        ),
        _ => false,
    }
}
