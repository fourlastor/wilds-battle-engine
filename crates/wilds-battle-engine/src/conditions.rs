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
    pub(crate) fn expire_action_conditions(&mut self, who: ParticipantId, move_id: Option<&str>) {
        self.get_mut(who).conditions.retain(|condition| !condition.rules.iter().any(|rule|
            matches!(rule, Rule::ExpiresOnAction {except_move} if move_id.is_none() || except_move.as_deref() != move_id)));
    }

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
        if let Scope::Pokemon(who) = scope
            && (condition.rules.iter().any(
                |r| matches!(r, Rule::InfatuatedBy(source) if !self.can_infatuate(*source, who)),
            ) || (matches!(
                condition.name.as_str(),
                "attract" | "disable" | "encore" | "heal_block" | "taunt" | "torment"
            ) && self.team_status_ability(who, "aromaveil"))
                || (condition.name == "taunt" && self.has_ability(who, "oblivious")))
        {
            return false;
        }
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

    pub(crate) fn explode_on_type(
        &mut self,
        actor: ParticipantId,
        move_type: PokemonType,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let rule = self.rules(actor).find_map(|r| match r {
            Rule::ExplodeOnType {
                move_type: kind,
                fraction,
                message,
            } if *kind == move_type => Some((*fraction, message.clone())),
            _ => None,
        });
        let Some((fraction, message)) = rule else {
            return false;
        };
        if let Some(message) = message {
            events.push(BattleEvent::Message(named(&message, &self.get(actor).name)));
        }
        if !self.has_ability(actor, "magicguard") {
            let damage = (f32::from(self.get(actor).max_hp) * fraction)
                .round()
                .max(1.0) as u16;
            self.damage(actor, damage, events);
        }
        true
    }

    pub(crate) fn swap_side_effects(&mut self, names: &[String]) -> bool {
        let mut swapped = [Vec::new(), Vec::new()];
        for side in Side::ALL {
            let (take, keep) = std::mem::take(&mut self.side_conditions[side.index()])
                .into_iter()
                .partition(|c| names.contains(&c.name));
            swapped[side.index()] = take;
            self.side_conditions[side.index()] = keep;
        }
        let changed = swapped.iter().any(|v| !v.is_empty());
        for side in Side::ALL {
            self.side_conditions[side.index()].append(&mut swapped[side.opposite().index()]);
        }
        changed
    }

    pub(crate) fn healing_blocked(&self, who: ParticipantId) -> bool {
        self.rules(who).any(|r| matches!(r, Rule::BlockHealing))
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

    pub(crate) fn swapped_defensive_stat(&self, stat: CombatStat) -> CombatStat {
        if self
            .field_conditions
            .iter()
            .flat_map(|c| &c.rules)
            .any(|r| matches!(r, Rule::SwapDefenses))
        {
            match stat {
                CombatStat::Defense => CombatStat::SpDefense,
                CombatStat::SpDefense => CombatStat::Defense,
                other => other,
            }
        } else {
            stat
        }
    }

    pub(crate) fn battle_raw_stat(&self, id: ParticipantId, stat: CombatStat) -> f32 {
        f32::from(self.get(id).raw_stat(self.swapped_defensive_stat(stat)))
    }

    pub(crate) fn battle_staged_stat(&self, id: ParticipantId, stat: CombatStat) -> f32 {
        // Battle Speed applies paralysis after all other modifiers, below.
        if stat != CombatStat::Speed && self.swapped_defensive_stat(stat) == stat {
            return self.get(id).effective_stat(stat);
        }
        let stage = f32::from(self.get(id).stage(stat.into()));
        let factor = if stage >= 0.0 {
            (stage + 2.0) / 2.0
        } else {
            2.0 / (2.0 - stage)
        };
        (self.battle_raw_stat(id, stat) * factor).trunc()
    }

    /// A stat with its stages and any multipliers from effects.
    pub(crate) fn stat_value(&self, id: ParticipantId, stat: CombatStat) -> f32 {
        let value = self.battle_staged_stat(id, stat);
        let factor = self.stat_factor(id, stat);
        let modified = (value * factor).trunc();
        if stat == CombatStat::Speed
            && self.get(id).status == Some(Status::Paralyzed)
            && !self.has_ability(id, "quickfeet")
        {
            (modified / 2.0).floor()
        } else {
            modified
        }
    }

    pub(crate) fn damage_taken_factor(
        &self,
        actor: ParticipantId,
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
                screen,
                unless_side_effect,
            } if only_category.is_none_or(|own| own == category)
                && only_type.is_none_or(|own| own == move_type)
                && !(*not_on_crit && critical)
                && !(*screen && self.has_ability(actor, "infiltrator"))
                && !unless_side_effect.as_ref().is_some_and(|name| {
                    self.side_conditions[id.side.index()]
                        .iter()
                        .any(|c| &c.name == name)
                }) =>
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

    pub(crate) fn blocks_grounded_status(&self, id: ParticipantId, status: Status) -> bool {
        self.on_ground(id) && self.get(id).hidden().is_none() && self.rules(id).any(|r| matches!(r, Rule::BlockGroundedStatus { statuses, .. } if statuses.is_empty() || statuses.contains(&status)))
    }

    pub(crate) fn blocks_status(&self, id: ParticipantId, status: Status) -> bool {
        self.rules(id).any(|rule| {
            matches!(rule, Rule::BlockStatus { statuses, .. }
                if statuses.is_empty() || statuses.contains(&status))
        })
    }

    pub(crate) fn blocks_grounded_confusion(&self, id: ParticipantId) -> bool {
        self.on_ground(id)
            && self.get(id).hidden().is_none()
            && self.rules(id).any(|r| {
                matches!(
                    r,
                    Rule::BlockGroundedStatus {
                        confusion: true,
                        ..
                    }
                )
            })
    }

    pub(crate) fn blocks_confusion(&self, id: ParticipantId) -> bool {
        if self.blocks_grounded_confusion(id) {
            return true;
        }
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

    fn matching_guard(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
        ignore_protect: bool,
    ) -> Option<&Rule> {
        if actor == target || matches!(spec.target, Target::Field | Target::All) {
            return None;
        }
        self.rules(target).filter(|rule| matches!(rule, Rule::Guard {spread_only, priority_only, damaging_only, status_only, unbypassable, ..}
            if (!ignore_protect || *unbypassable)
                && (!*spread_only || matches!(spec.target, Target::AllOthers | Target::AllOpponents))
                && (!*priority_only || self.effective_move_priority(actor,spec) > 0)
                && (!*damaging_only || spec.category != Category::Status)
                && (!*status_only || spec.category == Category::Status)))
            .max_by_key(|rule| matches!(rule,Rule::Guard{spread_only:true,..}|Rule::Guard{priority_only:true,..}))
    }

    pub(crate) fn guarded(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
        ignore_protect: bool,
    ) -> bool {
        self.matching_guard(actor, target, spec, ignore_protect)
            .is_some()
    }

    pub(crate) fn guard_hit(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
        options: &crate::moves::HitOptions,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let Some(Rule::Guard {
            contact_stat,
            contact_damage,
            contact_status,
            ..
        }) = self
            .matching_guard(actor, target, spec, options.ignore_protect)
            .cloned()
        else {
            return false;
        };
        if options.contact.unwrap_or(spec.has_flag("contact"))
            && !self.has_ability(actor, "longreach")
        {
            if let Some((stat, stages)) = contact_stat {
                self.change_stat_from(target, actor, stat, stages, events);
            }
            if let Some(status) = contact_status {
                self.inflict_status(
                    target,
                    actor,
                    status,
                    false,
                    PokemonType::None,
                    false,
                    events,
                );
            }
            if contact_damage > 0.0 && !self.has_ability(actor, "magicguard") {
                let amount =
                    ((f32::from(self.get(actor).max_hp) * contact_damage).floor() as u16).max(1);
                self.damage(actor, amount, events);
            }
        }
        true
    }

    pub(crate) fn break_protection(&mut self, who: ParticipantId) -> bool {
        let mut changed = std::mem::take(&mut self.get_mut(who).protected);
        for scope in [Scope::Pokemon(who), Scope::Side(who.side)] {
            let list = self.conditions_mut(scope);
            let before = list.len();
            list.retain(|condition| {
                !condition
                    .rules
                    .iter()
                    .any(|r| matches!(r, Rule::Guard { .. }))
            });
            changed |= before != list.len();
        }
        if changed {
            let pokemon = self.get_mut(who);
            pokemon.protection_streak = 0;
            pokemon.last_protection_turn = None;
        }
        changed
    }

    pub(crate) fn priority_blocked(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
    ) -> bool {
        actor.side != target.side && spec.target != Target::Field
            && self.effective_move_priority(actor, spec) > 0
            && self.rules(target).any(|rule| matches!(rule, Rule::BlockPriority {grounded_only} if !grounded_only || (self.on_ground(target) && self.get(target).hidden().is_none())))
    }

    pub(crate) fn effective_move_priority(&self, id: ParticipantId, spec: &MoveSpec) -> i8 {
        let bonus = spec.priority_terrain.as_ref().is_some_and(|name| {
            self.on_ground(id) && self.field_conditions.iter().any(|c| &c.name == name)
        });
        self.move_priority(id, spec.priority)
            .saturating_add(i8::from(bonus))
    }

    pub(crate) fn move_priority(&self, id: ParticipantId, base: i8) -> i8 {
        self.rules(id).fold(base, |priority, rule| match rule {
            Rule::MovePriority { amount } => priority.saturating_add(*amount),
            _ => priority,
        })
    }

    /// Called moves skip action-selection restrictions such as Disable and Taunt. Rules that
    /// prohibit executing a tagged move (Gravity, Heal Block) still apply.
    pub(crate) fn called_move_block(&self, id: ParticipantId, spec: &MoveSpec) -> Option<&str> {
        self.rules(id).find_map(|rule| match rule {
            Rule::BlockMoves { flags, message } if flags.iter().any(|flag| spec.has_flag(flag)) => {
                Some(message.as_deref().unwrap_or("{name} cannot use that move!"))
            }
            _ => None,
        })
    }

    pub(crate) fn only_move(&self, who: ParticipantId) -> Option<&str> {
        let p = self.get(who);
        self.rules(who).find_map(|rule| match rule {
            Rule::OnlyMove { id }
                if p.moves.contains(id) && p.move_pp.get(id).is_some_and(|pp| *pp > 0) =>
            {
                Some(id.as_str())
            }
            _ => None,
        })
    }

    pub(crate) fn move_block(&self, id: ParticipantId, spec: &MoveSpec) -> Option<&str> {
        if spec.id != "struggle"
            && self
                .only_move(id)
                .is_some_and(|required| required != spec.id)
        {
            return Some("{name} must repeat its encored move!");
        }
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

    pub(crate) fn floats_by_effect(&self, id: ParticipantId) -> bool {
        !self.is_grounded(id) && self.rules(id).any(|r| matches!(r, Rule::Airborne))
    }

    pub(crate) fn on_ground(&self, id: ParticipantId) -> bool {
        self.is_grounded(id)
            || (!self.floats_by_effect(id)
                && !self.types_of(id).contains(&PokemonType::Flying)
                && !self.has_ability(id, "levitate")
                && self.get(id).hidden() != Some(crate::model::HiddenKind::Air))
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

    pub(crate) fn flinch(&mut self, id: ParticipantId, secondary: bool) -> bool {
        if self.get(id).hp == 0
            || self.blocks_flinch(id)
            || self.has_ability(id, "innerfocus")
            || (secondary && self.has_ability(id, "shielddust"))
        {
            return false;
        }
        self.get_mut(id).flinched = true;
        true
    }

    pub(crate) fn incoming_base_power_factor(
        &self,
        id: ParticipantId,
        move_id: &str,
        kind: PokemonType,
    ) -> f32 {
        self.rules(id).fold(1.0, |factor, rule| match rule {
            Rule::IncomingBasePower {
                moves,
                move_type,
                factor: by,
                grounded_only,
            } if (moves.is_empty() || moves.iter().any(|name| name == move_id))
                && move_type.is_none_or(|t| t == kind)
                && (!grounded_only || (self.on_ground(id) && self.get(id).hidden().is_none())) =>
            {
                factor * by
            }
            _ => factor,
        })
    }

    pub(crate) fn base_power_factor(&self, id: ParticipantId, kind: PokemonType) -> f32 {
        self.rules(id).fold(1.0, |factor, rule| match rule {
            Rule::BasePower {
                move_type,
                factor: by,
                grounded_only,
            } if move_type.is_none_or(|own| own == kind)
                && (!grounded_only || self.on_ground(id)) =>
            {
                factor * by
            }
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
        let p = self.get_mut(id);
        p.conditions.retain(|condition| {
            !condition.rules.iter().any(|rule| {
                matches!(rule, Rule::RequiresStatus(required) if status != Some(*required)
                && !(*required == Status::Asleep && comatose))
                    || matches!(rule, Rule::OnlyMove {id} if !p.moves.contains(id) || p.move_pp.get(id).is_none_or(|pp| *pp == 0))
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
                    Rule::HealEachTurn {
                        fraction,
                        message,
                        grounded_only,
                    } => {
                        if grounded_only && (!self.on_ground(id) || self.get(id).hidden().is_some())
                        {
                            continue;
                        }
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
                    expired_rules.extend(
                        condition
                            .rules
                            .iter()
                            .cloned()
                            .map(|r| (condition.name.clone(), r)),
                    );
                    false
                }
                Some(turns) => {
                    *turns -= 1;
                    true
                }
                None => true,
            });
            if let Scope::Pokemon(id) = scope {
                for (condition_name, rule) in expired_rules {
                    if let Rule::StatusOnExpiry { status, source } = rule {
                        self.inflict_status(
                            // Safeguard is checked when Yawn starts, not at expiry.
                            if condition_name == "yawn" { id } else { source },
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
