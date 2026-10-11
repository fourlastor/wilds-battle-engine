//! Berry consumption effects. Hit-triggered activation is handled separately from eating.
use crate::engine::Battle;
use crate::items::item_key;
use crate::model::{
    BattleEvent, CombatStat, Condition, ParticipantId, PokemonType, Rule, Scope, Stat, Status,
};

#[derive(Clone, Copy)]
enum Berry {
    Heal(u16),
    HealFraction(u16),
    Flavor(CombatStat),
    Cure(Status),
    Poison,
    Confusion,
    Lum,
    Leppa,
    Boost(Stat),
    Lansat,
    Starf,
    Micle,
    Custap,
    Resist(PokemonType),
    Enigma,
    Reactive,
    Food,
}
fn berry(id: &str) -> Option<Berry> {
    use Berry::*;
    Some(match item_key(id).as_str() {
        "oranberry" => Heal(10),
        "sitrusberry" => HealFraction(4),
        "figyberry" => Flavor(CombatStat::Attack),
        "wikiberry" => Flavor(CombatStat::SpAttack),
        "magoberry" => Flavor(CombatStat::Speed),
        "aguavberry" => Flavor(CombatStat::SpDefense),
        "iapapaberry" => Flavor(CombatStat::Defense),
        "cheriberry" => Cure(Status::Paralyzed),
        "chestoberry" => Cure(Status::Asleep),
        "rawstberry" => Cure(Status::Burned),
        "aspearberry" => Cure(Status::Frozen),
        "pechaberry" => Poison,
        "persimberry" => Confusion,
        "lumberry" => Lum,
        "leppaberry" => Leppa,
        "liechiberry" => Boost(Stat::Attack),
        "ganlonberry" | "keeberry" => Boost(Stat::Defense),
        "petayaberry" => Boost(Stat::SpAttack),
        "apicotberry" | "marangaberry" => Boost(Stat::SpDefense),
        "salacberry" => Boost(Stat::Speed),
        "lansatberry" => Lansat,
        "starfberry" => Starf,
        "micleberry" => Micle,
        "custapberry" => Custap,
        "occaberry" => Resist(PokemonType::Fire),
        "passhoberry" => Resist(PokemonType::Water),
        "wacanberry" => Resist(PokemonType::Electric),
        "rindoberry" => Resist(PokemonType::Grass),
        "yacheberry" => Resist(PokemonType::Ice),
        "chopleberry" => Resist(PokemonType::Fighting),
        "kebiaberry" => Resist(PokemonType::Poison),
        "shucaberry" => Resist(PokemonType::Ground),
        "cobaberry" => Resist(PokemonType::Flying),
        "payapaberry" => Resist(PokemonType::Psychic),
        "tangaberry" => Resist(PokemonType::Bug),
        "chartiberry" => Resist(PokemonType::Rock),
        "kasibberry" => Resist(PokemonType::Ghost),
        "habanberry" => Resist(PokemonType::Dragon),
        "colburberry" => Resist(PokemonType::Dark),
        "babiriberry" => Resist(PokemonType::Steel),
        "chilanberry" => Resist(PokemonType::Normal),
        "roseliberry" => Resist(PokemonType::Fairy),
        "enigmaberry" => Enigma,
        "jabocaberry" | "rowapberry" => Reactive,
        "razzberry" | "blukberry" | "nanabberry" | "wepearberry" | "pinapberry" | "pomegberry"
        | "kelpsyberry" | "qualotberry" | "hondewberry" | "grepaberry" | "tamatoberry"
        | "cornnberry" | "magostberry" | "rabutaberry" | "nomelberry" | "spelonberry"
        | "pamtreberry" | "watmelberry" | "durinberry" | "belueberry" => Food,
        _ => return None,
    })
}

pub(crate) fn is_berry(id: &str) -> bool {
    berry(id).is_some()
}

impl Battle {
    pub fn has_eaten_berry(&self, who: ParticipantId) -> bool {
        self.get(who).ate_berry
    }

    fn berry_blocked(&self, who: ParticipantId) -> bool {
        !self.item_active(who)
            || self.actors().into_iter().any(|id| {
                id.side != who.side
                    && (self.has_ability(id, "unnerve")
                        || self.has_ability(id, "asoneglastrier")
                        || self.has_ability(id, "asonespectrier"))
            })
    }

    /// Forced consumption (Stuff Cheeks) bypasses held-item inhibition and Unnerve.
    pub(crate) fn eat_berry(
        &mut self,
        who: ParticipantId,
        forced: bool,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let Some(item) = self.get(who).item.clone() else {
            return false;
        };
        if !is_berry(&item) || self.get(who).hp == 0 || (!forced && self.berry_blocked(who)) {
            return false;
        }
        if !forced
            && self.healing_blocked(who)
            && matches!(
                berry(&item),
                Some(Berry::Heal(_) | Berry::HealFraction(_) | Berry::Flavor(_) | Berry::Enigma)
            )
        {
            return false;
        }
        self.get_mut(who).item = None;
        self.ability_lost_item(who);
        self.berry_effect(who, item, false, events);
        self.symbiosis_after_use(who);
        true
    }

    pub(crate) fn eat_stolen_berry(
        &mut self,
        from: ParticipantId,
        to: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if self.get(to).hp == 0 || !self.get(from).item.as_deref().is_some_and(is_berry) {
            return false;
        }
        let Some(item) = self.take_item(from, to) else {
            return false;
        };
        self.berry_effect(to, item, true, events);
        true
    }

    pub(crate) fn receive_berry(
        &mut self,
        who: ParticipantId,
        item: String,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if self.get(who).hp == 0 || !is_berry(&item) {
            return false;
        }
        self.berry_effect(who, item, true, events);
        true
    }

    fn berry_effect(
        &mut self,
        who: ParticipantId,
        item: String,
        stolen: bool,
        events: &mut Vec<BattleEvent>,
    ) {
        let Some(effect) = berry(&item) else {
            return;
        };
        let factor = if self.has_ability(who, "ripen") { 2 } else { 1 };
        let turn = self.turn;
        let p = self.get_mut(who);
        if !stolen {
            p.recyclable_item = Some(item.clone());
            p.item_used_turn = Some(turn);
        }
        p.last_consumed_item = Some(item.clone());
        p.consumed_items.insert(item.clone());
        p.ate_berry |= !stolen
            || !matches!(
                effect,
                Berry::Resist(_) | Berry::Enigma | Berry::Reactive | Berry::Custap | Berry::Food
            );
        events.push(BattleEvent::Message(format!("{} ate {item}!", p.name)));
        let heal = match effect {
            Berry::Heal(amount) => amount,
            Berry::HealFraction(divisor) => (p.max_hp / divisor).max(1),
            Berry::Flavor(_) => (p.max_hp / 3).max(1),
            _ => 0,
        };
        if heal > 0 {
            self.heal(who, heal.saturating_mul(factor), events);
        }
        match effect {
            Berry::Flavor(stat) if self.get(who).nature_down == Some(stat) => {
                self.confuse(who, who, events);
            }
            Berry::Cure(status) => {
                self.cure_status(who, Some(status), events);
            }
            Berry::Poison
                if matches!(
                    self.get(who).status,
                    Some(Status::Poisoned | Status::BadlyPoisoned)
                ) =>
            {
                self.cure_status(who, None, events);
            }
            Berry::Lum => {
                self.cure_status(who, None, events);
                self.get_mut(who).confused_turns = None;
            }
            Berry::Confusion => {
                self.get_mut(who).confused_turns = None;
            }
            Berry::Boost(stat) => {
                self.change_stat(who, stat, factor as i8, events);
            }
            Berry::Lansat => {
                self.get_mut(who).crit_stage = self.get(who).crit_stage.max(2);
            }
            Berry::Starf => {
                let stats: Vec<_> = [
                    Stat::Attack,
                    Stat::Defense,
                    Stat::SpAttack,
                    Stat::SpDefense,
                    Stat::Speed,
                ]
                .into_iter()
                .filter(|stat| self.get(who).stage(*stat) < 6)
                .collect();
                if !stats.is_empty() {
                    let index = self.rng.range(0, stats.len() as u8 - 1) as usize;
                    self.change_stat(who, stats[index], 2 * factor as i8, events);
                }
            }
            Berry::Leppa => {
                let moves = self.get(who).moves.clone();
                let chosen = moves
                    .iter()
                    .find(|id| self.get(who).move_pp.get(*id) == Some(&0))
                    .or_else(|| {
                        moves.iter().find(|id| {
                            self.get(who)
                                .move_pp
                                .get(*id)
                                .is_some_and(|pp| self.max_pp(who, id).is_some_and(|max| *pp < max))
                        })
                    });
                if let Some(id) = chosen {
                    let max = self.max_pp(who, id).expect("known move");
                    let pp = self.get_mut(who).move_pp.get_mut(id).expect("known PP");
                    *pp = pp.saturating_add(10 * factor as u8).min(max);
                }
            }
            Berry::Micle => {
                self.start_condition(
                    Scope::Pokemon(who),
                    Condition {
                        name: "micle_berry".into(),
                        value: 1,
                        turns_left: Some(2),
                        group: None,
                        end_message: None,
                        rules: vec![Rule::AccuracyMultiplier {
                            factor: 4915.0 / 4096.0,
                        }],
                    },
                    true,
                );
            }
            // These berries have effects only when their held-item trigger fires.
            Berry::Resist(kind) => {
                let _ = kind;
            }
            _ => {}
        }
        if self.has_ability(who, "cheekpouch") {
            self.heal(who, (self.get(who).max_hp / 3).max(1), events);
        }
    }

    pub(crate) fn update_berries(&mut self, events: &mut Vec<BattleEvent>) {
        self.update_status_abilities(events);
        for who in self.actors() {
            let p = self.get(who);
            let Some(item) = p.item.as_deref() else {
                continue;
            };
            let Some(effect) = berry(item) else {
                continue;
            };
            let low = p.hp
                <= p.max_hp
                    / if self.has_ability(who, "gluttony") {
                        2
                    } else {
                        4
                    };
            let trigger = match effect {
                Berry::Heal(_) | Berry::HealFraction(_) => p.hp <= p.max_hp / 2,
                Berry::Flavor(_) | Berry::Lansat | Berry::Starf => low,
                Berry::Boost(_) => {
                    low && !matches!(item_key(item).as_str(), "keeberry" | "marangaberry")
                }
                Berry::Cure(status) => p.status == Some(status),
                Berry::Poison => matches!(p.status, Some(Status::Poisoned | Status::BadlyPoisoned)),
                Berry::Lum => p.status.is_some() || p.confused_turns.is_some(),
                Berry::Confusion => p.confused_turns.is_some(),
                Berry::Leppa => p.move_pp.values().any(|pp| *pp == 0),
                _ => false,
            };
            if trigger {
                self.eat_berry(who, false, events);
            }
        }
    }
}

impl Battle {
    pub(crate) fn resistance_berry_factor(
        &mut self,
        target: ParticipantId,
        kind: PokemonType,
        effectiveness: f32,
        events: &mut Vec<BattleEvent>,
    ) -> f32 {
        let Some(Berry::Resist(resists)) = self.get(target).item.as_deref().and_then(berry) else {
            return 1.0;
        };
        if resists != kind || (kind != PokemonType::Normal && effectiveness <= 1.0) {
            return 1.0;
        }
        let ripen = self.has_ability(target, "ripen");
        if self.eat_berry(target, false, events) {
            if ripen { 0.25 } else { 0.5 }
        } else {
            1.0
        }
    }

    pub(crate) fn berry_hit(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        meta: &crate::engine::HitMeta,
        events: &mut Vec<BattleEvent>,
    ) {
        let Some(item) = self.get(target).item.as_deref().map(item_key) else {
            return;
        };
        if item == "enigmaberry"
            && meta
                .effectiveness
                .unwrap_or_else(|| self.effectiveness(target, meta.move_type))
                > 1.0
        {
            let factor = if self.has_ability(target, "ripen") {
                2
            } else {
                1
            };
            if self.eat_berry(target, false, events) {
                self.heal(
                    target,
                    (self.get(target).max_hp / 4).max(1) * factor,
                    events,
                );
            }
        } else if ((item == "jabocaberry" && meta.category == crate::moves::Category::Physical)
            || (item == "rowapberry" && meta.category == crate::moves::Category::Special))
            && self.get(actor).hp > 0
            && !self.has_ability(actor, "magicguard")
        {
            let divisor = if self.has_ability(target, "ripen") {
                4
            } else {
                8
            };
            if self.eat_berry(target, false, events) {
                self.damage(actor, (self.get(actor).max_hp / divisor).max(1), events);
            }
        }
    }

    pub(crate) fn finish_item_hits(&mut self, events: &mut Vec<BattleEvent>) {
        for (actor, target, meta) in std::mem::take(&mut self.item_hits) {
            if meta.defer_items {
                self.berry_hit(actor, target, &meta, events);
            }
            if self.sheer_force_applies(actor, &meta.move_id) {
                continue;
            }
            let Some(item) = self.get(target).item.as_deref().map(item_key) else {
                continue;
            };
            if (item == "keeberry" && meta.category == crate::moves::Category::Physical)
                || (item == "marangaberry" && meta.category == crate::moves::Category::Special)
            {
                self.eat_berry(target, false, events);
            }
        }
    }

    pub(crate) fn custap_priority(
        &mut self,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let p = self.get(who);
        let low = p.hp
            <= p.max_hp
                / if self.has_ability(who, "gluttony") {
                    2
                } else {
                    4
                };
        low && p
            .item
            .as_deref()
            .is_some_and(|id| item_key(id) == "custapberry")
            && self.eat_berry(who, false, events)
    }

    pub(crate) fn residual_berries(&mut self, events: &mut Vec<BattleEvent>) {
        for who in self.actors() {
            let p = self.get(who);
            let low = p.hp
                <= p.max_hp
                    / if self.has_ability(who, "gluttony") {
                        2
                    } else {
                        4
                    };
            if low
                && p.item
                    .as_deref()
                    .is_some_and(|id| item_key(id) == "micleberry")
            {
                self.eat_berry(who, false, events);
            }
        }
    }
}
