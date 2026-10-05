//! Marks and timed effects on a Pokémon, a side or the field, and the rules they switch on.
//!
//! Moves set these through the script API. The engine keeps no list of known effects: a move
//! names its own ("reflect", "minimized") and picks the rules it needs.
use crate::engine::Battle;
use crate::model::{
    BattleEvent, CombatStat, Condition, ParticipantId, PokemonType, Rule, Scope, Side, Status,
};
use crate::moves::Category;

/// Fills in `{name}` in a message written by a move.
pub(crate) fn named(message: &str, name: &str) -> String {
    message.replace("{name}", name)
}

impl Battle {
    pub(crate) fn conditions(&self, scope: Scope) -> &[Condition] {
        match scope {
            Scope::Pokemon(id) => &self.get(id).conditions,
            Scope::Side(side) => &self.side_conditions[side.index()],
            Scope::Field => &self.field_conditions,
        }
    }
    fn conditions_mut(&mut self, scope: Scope) -> &mut Vec<Condition> {
        match scope {
            Scope::Pokemon(id) => &mut self.get_mut(id).conditions,
            Scope::Side(side) => &mut self.side_conditions[side.index()],
            Scope::Field => &mut self.field_conditions,
        }
    }

    /// Every rule that applies to a Pokémon: its own, its side's and the field's.
    pub(crate) fn rules(&self, id: ParticipantId) -> impl Iterator<Item = &Rule> {
        self.get(id)
            .conditions
            .iter()
            .chain(self.side_conditions[id.side.index()].iter())
            .chain(self.field_conditions.iter())
            .flat_map(|condition| condition.rules.iter())
    }

    /// Sets a mark, or changes the value of a condition that is already there.
    pub(crate) fn set_mark(&mut self, scope: Scope, name: &str, value: i32, turns: Option<u8>) {
        let conditions = self.conditions_mut(scope);
        if let Some(existing) = conditions.iter_mut().find(|own| own.name == name) {
            existing.value = value;
            if existing.rules.is_empty() {
                existing.turns_left = turns;
            }
        } else {
            conditions.push(Condition {
                name: name.to_owned(),
                value,
                turns_left: turns,
                rules: Vec::new(),
                group: None,
                end_message: None,
            });
        }
    }

    /// Starts a timed effect. Returns false if it is already there and `restart` is not set.
    pub(crate) fn start_condition(
        &mut self,
        scope: Scope,
        condition: Condition,
        restart: bool,
    ) -> bool {
        let conditions = self.conditions_mut(scope);
        if conditions.iter().any(|own| own.name == condition.name) {
            if !restart {
                return false;
            }
            conditions.retain(|own| own.name != condition.name);
        }
        if let Some(group) = &condition.group {
            conditions.retain(|own| own.group.as_ref() != Some(group));
        }
        conditions.push(condition);
        true
    }

    pub(crate) fn end_condition(&mut self, scope: Scope, name: &str) -> bool {
        let conditions = self.conditions_mut(scope);
        let before = conditions.len();
        conditions.retain(|own| own.name != name);
        conditions.len() != before
    }

    pub(crate) fn end_group(&mut self, scope: Scope, group: &str) -> bool {
        let conditions = self.conditions_mut(scope);
        let before = conditions.len();
        conditions.retain(|own| own.group.as_deref() != Some(group));
        conditions.len() != before
    }

    pub(crate) fn stat_factor(&self, id: ParticipantId, stat: CombatStat) -> f32 {
        self.rules(id).fold(1.0, |factor, rule| match rule {
            Rule::StatMultiplier {
                stat: own,
                factor: by,
            } if *own == stat => factor * by,
            _ => factor,
        })
    }

    /// A stat with its stages and any multipliers from effects.
    pub(crate) fn stat_value(&self, id: ParticipantId, stat: CombatStat) -> f32 {
        let value = self.get(id).effective_stat(stat);
        let factor = self.stat_factor(id, stat);
        if factor == 1.0 {
            value
        } else {
            (value * factor).trunc()
        }
    }

    pub(crate) fn damage_taken_factor(
        &self,
        id: ParticipantId,
        category: Category,
        move_type: PokemonType,
        critical: bool,
    ) -> f32 {
        self.rules(id).fold(1.0, |factor, rule| match rule {
            Rule::DamageTaken {
                category: only_category,
                move_type: only_type,
                factor: by,
                not_on_crit,
            } if only_category.is_none_or(|own| own == category)
                && only_type.is_none_or(|own| own == move_type)
                && !(*not_on_crit && critical) =>
            {
                factor * by
            }
            _ => factor,
        })
    }

    pub(crate) fn damage_dealt_factor(
        &self,
        id: ParticipantId,
        category: Category,
        move_type: PokemonType,
    ) -> f32 {
        self.rules(id).fold(1.0, |factor, rule| match rule {
            Rule::DamageDealt {
                category: only_category,
                move_type: only_type,
                factor: by,
            } if only_category.is_none_or(|own| own == category)
                && only_type.is_none_or(|own| own == move_type) =>
            {
                factor * by
            }
            _ => factor,
        })
    }

    pub(crate) fn blocks_status(&self, id: ParticipantId, status: Status) -> bool {
        self.rules(id).any(|rule| {
            matches!(rule, Rule::BlockStatus { statuses, .. }
                if statuses.is_empty() || statuses.contains(&status))
        })
    }

    pub(crate) fn blocks_confusion(&self, id: ParticipantId) -> bool {
        self.rules(id).any(|rule| {
            matches!(
                rule,
                Rule::BlockStatus {
                    confusion: true,
                    ..
                }
            )
        })
    }

    pub(crate) fn blocks_stat_drops(&self, id: ParticipantId) -> bool {
        self.rules(id)
            .any(|rule| matches!(rule, Rule::BlockStatDrops))
    }

    /// A chance of an extra effect of one of `id`'s moves, after any multipliers.
    pub(crate) fn effect_chance(&self, id: ParticipantId, chance: f32) -> f32 {
        self.rules(id)
            .fold(chance, |chance, rule| match rule {
                Rule::EffectChance { factor } => chance * factor,
                _ => chance,
            })
            .min(1.0)
    }

    /// The type a move of `move_type` has when `id` uses it.
    pub(crate) fn converted_type(&self, id: ParticipantId, move_type: PokemonType) -> PokemonType {
        self.rules(id)
            .find_map(|rule| match rule {
                Rule::MoveType { from, to } if *from == move_type => Some(*to),
                _ => None,
            })
            .unwrap_or(move_type)
    }

    pub(crate) fn is_grounded(&self, id: ParticipantId) -> bool {
        self.rules(id).any(|rule| matches!(rule, Rule::Grounded))
    }

    pub(crate) fn is_trapped(&self, id: ParticipantId) -> bool {
        self.rules(id).any(|rule| matches!(rule, Rule::Trapped))
    }

    pub(crate) fn endures(&self, id: ParticipantId) -> bool {
        self.rules(id).any(|rule| matches!(rule, Rule::Endure))
    }

    pub(crate) fn always_hits(&self, actor: ParticipantId, target: ParticipantId) -> bool {
        self.rules(actor).any(|rule| {
            matches!(rule, Rule::AlwaysHit { against }
                if against.is_none_or(|only| only == target))
        })
    }

    /// The types a Pokémon counts as having right now.
    pub(crate) fn types_of(&self, id: ParticipantId) -> Vec<PokemonType> {
        let mut types = self.get(id).types.clone();
        for rule in self.rules(id) {
            if let Rule::WithoutType(ignored) = rule {
                types.retain(|own| own != ignored);
            }
        }
        types
    }

    /// The types that count when a move of `move_type` is used on a Pokémon.
    pub(crate) fn defending_types(
        &self,
        id: ParticipantId,
        move_type: PokemonType,
    ) -> Vec<PokemonType> {
        let mut types = self.types_of(id);
        if move_type == PokemonType::Ground && self.is_grounded(id) {
            types.retain(|own| *own != PokemonType::Flying);
        }
        types
    }

    /// End of turn: what timed effects do each turn, then their countdown.
    pub(crate) fn tick_conditions(
        &mut self,
        order: &[ParticipantId],
        events: &mut Vec<BattleEvent>,
    ) {
        for &id in order {
            if self.winner.is_some() {
                return;
            }
            let each_turn: Vec<Rule> = self
                .rules(id)
                .filter(|rule| {
                    matches!(
                        rule,
                        Rule::DamageEachTurn { .. }
                            | Rule::HealEachTurn { .. }
                            | Rule::DrainEachTurn { .. }
                    )
                })
                .cloned()
                .collect();
            for rule in each_turn {
                if self.get(id).hp == 0 {
                    break;
                }
                let name = self.get(id).name.clone();
                let max_hp = self.get(id).max_hp;
                let share = |fraction: f32| ((f32::from(max_hp) * fraction).floor() as u16).max(1);
                match rule {
                    Rule::DamageEachTurn {
                        fraction,
                        except_types,
                        message,
                    } => {
                        let types = self.types_of(id);
                        if except_types.iter().any(|spared| types.contains(spared)) {
                            continue;
                        }
                        if let Some(message) = message {
                            events.push(BattleEvent::Message(named(&message, &name)));
                        }
                        self.damage(id, share(fraction), events);
                    }
                    Rule::HealEachTurn { fraction, message } => {
                        if self.get(id).hp < max_hp {
                            self.heal(id, share(fraction), events);
                            if let Some(message) = message {
                                events.push(BattleEvent::Message(named(&message, &name)));
                            }
                        }
                    }
                    Rule::DrainEachTurn {
                        fraction,
                        to,
                        message,
                    } => {
                        let amount = share(fraction).min(self.get(id).hp);
                        if let Some(message) = message {
                            events.push(BattleEvent::Message(named(&message, &name)));
                        }
                        self.damage(id, amount, events);
                        if self.get(to).hp > 0 && self.get(to).hp < self.get(to).max_hp {
                            self.heal(to, amount, events);
                        }
                    }
                    _ => {}
                }
            }
            self.resolve_outcome(events);
        }
        if self.winner.is_some() {
            return;
        }
        let mut scopes = vec![
            Scope::Field,
            Scope::Side(Side::Allies),
            Scope::Side(Side::Foes),
        ];
        scopes.extend(order.iter().map(|&id| Scope::Pokemon(id)));
        for scope in scopes {
            let name = match scope {
                Scope::Pokemon(id) => self.get(id).name.clone(),
                _ => String::new(),
            };
            let conditions = self.conditions_mut(scope);
            let mut ended = Vec::new();
            conditions.retain_mut(|condition| match &mut condition.turns_left {
                Some(turns) if *turns <= 1 => {
                    ended.extend(condition.end_message.take());
                    false
                }
                Some(turns) => {
                    *turns -= 1;
                    true
                }
                None => true,
            });
            for message in ended {
                events.push(BattleEvent::Message(named(&message, &name)));
            }
        }
    }
}
