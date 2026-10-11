//! Marks and timed effects on a Pokémon, a side or the field, and the rules they switch on.
//!
//! Moves set these through the script API. The engine keeps no list of known effects: a move
//! names its own ("reflect", "minimized") and picks the rules it needs.
use crate::engine::Battle;
use crate::model::{
    BattleEvent, CombatStat, Condition, ParticipantId, PokemonType, Rule, Scope, Side, Status,
};
use crate::moves::{Category, MoveSpec, Target};

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
            .chain(
                self.get(id)
                    .ability_rules
                    .iter()
                    .filter(move |_| self.ability_active(id)),
            )
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

    pub(crate) fn accuracy_rule_factor(&self, id: ParticipantId) -> f32 {
        self.rules(id).fold(1.0, |factor, rule| match rule {
            Rule::AccuracyMultiplier { factor: by } => factor * by,
            _ => factor,
        })
    }

    pub(crate) fn action_speed(&self, id: ParticipantId) -> i32 {
        let speed = (self.stat_value(id, CombatStat::Speed) as i32).clamp(0, 10000);
        let speed = if self
            .field_conditions
            .iter()
            .flat_map(|c| &c.rules)
            .any(|r| matches!(r, Rule::ReverseSpeed))
        {
            10000 - speed
        } else {
            speed
        };
        speed & 8191
    }

    pub(crate) fn stat_factor(&self, id: ParticipantId, stat: CombatStat) -> f32 {
        self.rules(id).fold(
            self.ability_stat_factor(id, stat),
            |factor, rule| match rule {
                Rule::StatMultiplier {
                    stat: own,
                    factor: by,
                } if *own == stat => factor * by,
                _ => factor,
            },
        )
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
                multi_battle_factor,
            } if only_category.is_none_or(|own| own == category)
                && only_type.is_none_or(|own| own == move_type)
                && !(*not_on_crit && critical) =>
            {
                // Fainting does not change the battle format.
                let multi = self.active.iter().any(|side| side.len() > 1);
                factor
                    * if multi {
                        multi_battle_factor.unwrap_or(*by)
                    } else {
                        *by
                    }
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
        ["clearbody", "whitesmoke", "fullmetalbody"]
            .iter()
            .any(|ability| self.has_ability(id, ability))
            || self
                .rules(id)
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

    pub(crate) fn redirect_target(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
    ) -> ParticipantId {
        if spec.has_flag("ignore_redirection")
            || !matches!(
                self.move_target_kind(actor, spec),
                Target::Selected | Target::AnyOther | Target::UserOrAlly | Target::RandomOpponent
            )
            || actor == target
        {
            return target;
        }
        let move_type = self.converted_type(actor, spec.move_type);
        self.rules(target)
            .enumerate()
            .filter_map(|(order, rule)| {
                let (to, priority) = match rule {
                    Rule::Redirect {
                        to,
                        move_type: only_type,
                        priority,
                    } if only_type.is_none_or(|kind| kind == move_type) => (*to, *priority),
                    Rule::Attention {
                        to,
                        priority,
                        powder,
                        allow_carried,
                    } if actor.side != to.side
                        && (*allow_carried || self.get(*to).carried_by.is_none())
                        && (!powder
                            || (!self.types_of(actor).contains(&PokemonType::Grass)
                                && !self.has_ability(actor, "overcoat"))) =>
                    {
                        (*to, *priority)
                    }
                    _ => return None,
                };
                (to.side == target.side
                    && to != actor
                    && self.get(to).hp > 0
                    && self.is_active(to)
                    && self.adjacent(actor, to))
                .then_some((priority, order, to))
            })
            .max_by_key(|(priority, order, _)| (*priority, *order))
            .map_or(target, |(_, _, to)| to)
    }

    pub(crate) fn move_priority(&self, id: ParticipantId, base: i8) -> i8 {
        self.rules(id).fold(base, |priority, rule| match rule {
            Rule::MovePriority { amount } => priority.saturating_add(*amount),
            _ => priority,
        })
    }

    pub(crate) fn move_block(&self, id: ParticipantId, spec: &MoveSpec) -> Option<&str> {
        if spec.has_flag("requires_eaten_berry") && !self.has_eaten_berry(id) {
            return Some("{name} must eat a berry before using that move!");
        }
        if spec.has_flag("requires_held_berry")
            && !self
                .get(id)
                .item
                .as_deref()
                .is_some_and(crate::berries::is_berry)
        {
            return Some("{name} needs to hold a berry to use that move!");
        }
        if spec.id != "struggle"
            && self.actors().into_iter().any(|source| {
                source.side != id.side
                    && self.get(source).moves.contains(&spec.id)
                    && self
                        .get(source)
                        .conditions
                        .iter()
                        .flat_map(|c| &c.rules)
                        .any(|r| matches!(r, Rule::SealMoves))
            })
        {
            return Some("{name}'s move is sealed by Imprison!");
        }
        self.rules(id).find_map(|rule| match rule {
            Rule::BlockMove {
                id: blocked,
                message,
            } if *blocked == spec.id => Some(
                message
                    .as_deref()
                    .unwrap_or("{name} cannot use the disabled move!"),
            ),
            Rule::BlockCategory {
                category,
                except,
                message,
            } if *category == spec.category && !except.contains(&spec.id) => Some(
                message
                    .as_deref()
                    .unwrap_or("{name} cannot use that category of move!"),
            ),
            Rule::BlockMoves { flags, message } if flags.iter().any(|flag| spec.has_flag(flag)) => {
                Some(message.as_deref().unwrap_or("{name} cannot use that move!"))
            }
            _ => None,
        })
    }

    pub(crate) fn is_grounded(&self, id: ParticipantId) -> bool {
        self.rules(id).any(|rule| matches!(rule, Rule::Grounded))
    }

    pub(crate) fn is_trapped(&self, id: ParticipantId) -> bool {
        self.rules(id).any(|rule| matches!(rule, Rule::Trapped)) || self.linked_trap(id)
    }

    pub(crate) fn linked_trap(&self, id: ParticipantId) -> bool {
        self.rules(id).any(|rule| {
            matches!(rule, Rule::TrappedBy { source }
            if self.is_active(*source) && self.get(*source).hp > 0)
        })
    }

    pub(crate) fn blocks_flinch(&self, id: ParticipantId) -> bool {
        self.rules(id).any(|rule| matches!(rule, Rule::BlockFlinch))
    }

    pub(crate) fn flinch(&mut self, id: ParticipantId) -> bool {
        if self.get(id).hp == 0 || self.blocks_flinch(id) {
            return false;
        }
        self.get_mut(id).flinched = true;
        true
    }

    pub(crate) fn base_power_factor(&self, id: ParticipantId, kind: PokemonType) -> f32 {
        self.rules(id).fold(1.0, |factor, rule| match rule {
            Rule::BasePower {
                move_type,
                factor: by,
            } if move_type.is_none_or(|own| own == kind) => factor * by,
            _ => factor,
        })
    }

    pub(crate) fn effectiveness_factor(&self, id: ParticipantId, kind: PokemonType) -> f32 {
        self.rules(id).fold(1.0, |factor, rule| match rule {
            Rule::TypeEffectiveness {
                move_type,
                factor: by,
            } if *move_type == kind => factor * by,
            _ => factor,
        })
    }

    pub(crate) fn infatuation_prevents_move(&mut self, id: ParticipantId) -> bool {
        if self.has_ability(id, "oblivious") {
            self.get_mut(id).conditions.retain(|condition| {
                !condition
                    .rules
                    .iter()
                    .any(|rule| matches!(rule, Rule::InfatuatedBy(_)))
            });
            return false;
        }
        let infatuated = self.rules(id).any(|rule| {
            matches!(rule, Rule::InfatuatedBy(by)
            if self.get(*by).hp > 0 && self.is_active(*by))
        });
        infatuated && self.rng.chance(0.5)
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
        if let Some(added) = self.get(id).added_type
            && !types.contains(&added)
        {
            types.push(added);
        }
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
        for rule in self.rules(id) {
            if let Rule::NegateTypeImmunity {
                defending,
                incoming,
            } = rule
                && incoming.contains(&move_type)
            {
                types.retain(|kind| kind != defending);
            }
        }
        if move_type == PokemonType::Ground && self.is_grounded(id) {
            types.retain(|own| *own != PokemonType::Flying);
        }
        types
    }

    pub(crate) fn remove_expired_status_conditions(&mut self, id: ParticipantId) {
        let status = self.get(id).status;
        let comatose = self.has_ability(id, "comatose");
        self.get_mut(id).conditions.retain(|condition| {
            !condition.rules.iter().any(|rule| {
                matches!(rule, Rule::RequiresStatus(required) if status != Some(*required)
                && !(*required == Status::Asleep && comatose))
            })
        });
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
            self.remove_expired_status_conditions(id);
            let each_turn: Vec<Rule> = self
                .rules(id)
                .filter(|rule| {
                    matches!(
                        rule,
                        Rule::StatEachTurn { .. }
                            | Rule::DamageEachTurn { .. }
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
                    Rule::StatEachTurn {
                        stat,
                        stages,
                        source,
                    } => {
                        if self.is_active(source) && self.get(source).hp > 0 {
                            self.change_stat_from(source, id, stat, stages, events);
                        }
                    }
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
        let mut perish = Vec::new();
        for scope in scopes {
            let name = match scope {
                Scope::Pokemon(id) => self.get(id).name.clone(),
                _ => String::new(),
            };
            if let Scope::Pokemon(id) = scope {
                for condition in self.conditions(scope) {
                    if condition.rules.contains(&Rule::FaintOnExpiry)
                        && let Some(turns) = condition.turns_left
                    {
                        events.push(BattleEvent::Message(format!(
                            "{}'s perish count fell to {}!",
                            name,
                            turns - 1
                        )));
                        if turns == 1 {
                            perish.push(id);
                        }
                    }
                }
            }
            let conditions = self.conditions_mut(scope);
            let mut ended = Vec::new();
            let mut expired_rules = Vec::new();
            conditions.retain_mut(|condition| match &mut condition.turns_left {
                Some(turns) if *turns <= 1 => {
                    ended.extend(condition.end_message.take());
                    expired_rules.extend(condition.rules.clone());
                    false
                }
                Some(turns) => {
                    *turns -= 1;
                    true
                }
                None => true,
            });
            if let Scope::Pokemon(id) = scope {
                for rule in expired_rules {
                    if let Rule::StatusOnExpiry { status, source } = rule {
                        self.inflict_status(
                            source,
                            id,
                            status,
                            false,
                            PokemonType::None,
                            false,
                            events,
                        );
                    }
                }
            }
            for message in ended {
                events.push(BattleEvent::Message(named(&message, &name)));
            }
        }
        // Perish countdowns expire in Speed order, independent of the moves' priorities.
        perish.sort_by_key(|&id| std::cmp::Reverse(self.action_speed(id)));
        for id in perish {
            if self.winner.is_some() {
                break;
            }
            if self.get(id).hp > 0 {
                self.damage(id, self.get(id).hp, events);
            }
            self.resolve_outcome(events);
        }
    }
}
