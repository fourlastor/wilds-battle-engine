//! Runs the Lua hooks of scripted moves: what a script can read (`ctx`) and what each operation
//! it yields does to the battle.
use crate::engine::{Attack, Battle, HitMeta, HitReaction};
use crate::model::{
    AppliedStatus, BattleError, BattleEvent, Bound, CombatStat, HiddenKind, NextMovePowerBoost,
    ParticipantId, Pokemon, Scope, ScriptContinuation, Side, Stat, Status, Weather,
};
use crate::moves::{
    DamageOptions, HitOptions, HpAmount, MoveSpec, ScopeRef, ScriptHook, ScriptOperation, Target,
};
use crate::type_chart;
use mlua::thread::ThreadStatus;
use mlua::{Lua, Table, Thread, Value};

/// How many operations one hook may run before the engine gives up on it.
pub(crate) const MAX_OPERATIONS: u32 = 256;

/// One run of a hook: who is using which move on whom.
struct Frame<'a> {
    actor: ParticipantId,
    target: ParticipantId,
    spec: &'a MoveSpec,
    hook: ScriptHook,
    any_hit: bool,
    /// A hit was tried and did not land, or the script said the move failed.
    failed: bool,
    /// What `try_hit` found for a Pokémon, until the next hit on it uses the answer.
    checked: Vec<(ParticipantId, bool)>,
}

impl Frame<'_> {
    /// The answer `try_hit` gave for a Pokémon, if no hit has used it yet.
    fn take_check(&mut self, who: ParticipantId) -> Option<bool> {
        let position = self.checked.iter().position(|(own, _)| *own == who)?;
        Some(self.checked.remove(position).1)
    }
}

/// The Lua side of a running hook. The tables are refreshed after every operation.
struct Run {
    thread: Thread,
    ctx: Table,
    pokemon: Vec<(ParticipantId, Table)>,
    sides: [Table; 2],
    field: Table,
}

enum Flow {
    Continue(Value),
    Stop,
}

/// What became of one attempt to hit a Pokémon.
#[derive(Clone, Copy, PartialEq, Eq)]
enum HitOutcome {
    Hit,
    Fainted,
    Protected,
    Hidden,
    Missed,
}

#[derive(Default)]
struct DamageReport {
    hit: bool,
    damage: u16,
    fainted: bool,
    critical: bool,
    effectiveness: f32,
    missed: bool,
    blocked: bool,
    immune: bool,
    hits: u8,
    /// Who the hit landed on, and who it did not reach.
    reached: Vec<ParticipantId>,
    unreached: Vec<ParticipantId>,
}

fn symbol(lua: &Lua, table: &str, name: &str) -> mlua::Result<Value> {
    lua.globals().get::<Table>(table)?.get::<Value>(name)
}

fn side_name(side: Side) -> &'static str {
    match side {
        Side::Allies => "allies",
        Side::Foes => "foes",
    }
}

impl Battle {
    fn pokemon_table(run: &Run, id: ParticipantId) -> Option<&Table> {
        run.pokemon
            .iter()
            .find(|(own, _)| *own == id)
            .map(|(_, table)| table)
    }

    fn conditions_tables(&self, lua: &Lua, scope: Scope) -> mlua::Result<(Table, Table)> {
        let marks = lua.create_table()?;
        let effects = lua.create_table()?;
        for condition in self.conditions(scope) {
            marks.raw_set(condition.name.as_str(), condition.value)?;
            match condition.turns_left {
                Some(turns) => effects.raw_set(condition.name.as_str(), turns)?,
                None => effects.raw_set(condition.name.as_str(), true)?,
            }
        }
        Ok((marks, effects))
    }

    fn fill_pokemon(
        &self,
        lua: &Lua,
        run: &Run,
        frame: &Frame,
        id: ParticipantId,
        table: &Table,
    ) -> mlua::Result<()> {
        let pokemon: &Pokemon = self.get(id);
        table.raw_set("name", pokemon.name.as_str())?;
        table.raw_set("species", pokemon.species.as_str())?;
        table.raw_set("level", pokemon.level)?;
        table.raw_set("hp", pokemon.hp)?;
        table.raw_set("max_hp", pokemon.max_hp)?;
        table.raw_set(
            "status",
            match pokemon.status {
                Some(status) => symbol(lua, "Status", &format!("{status:?}"))?,
                None => Value::Nil,
            },
        )?;
        let types = lua.create_table()?;
        for kind in self.types_of(id) {
            types.raw_push(symbol(lua, "Type", &format!("{kind:?}"))?)?;
        }
        table.raw_set("types", types)?;
        let stages = lua.create_table()?;
        for stat in [
            Stat::Attack,
            Stat::Defense,
            Stat::SpAttack,
            Stat::SpDefense,
            Stat::Speed,
            Stat::Accuracy,
            Stat::Evasion,
        ] {
            stages.raw_set(
                symbol(lua, "Stat", &format!("{stat:?}"))?,
                pokemon.stage(stat),
            )?;
        }
        table.raw_set("stages", stages)?;
        for (key, stat) in [
            ("attack", CombatStat::Attack),
            ("defense", CombatStat::Defense),
            ("sp_attack", CombatStat::SpAttack),
            ("sp_defense", CombatStat::SpDefense),
            ("speed", CombatStat::Speed),
        ] {
            table.raw_set(key, self.stat_value(id, stat) as i64)?;
        }
        table.raw_set(
            "gender",
            symbol(lua, "Gender", &format!("{:?}", pokemon.gender))?,
        )?;
        table.raw_set("weight", pokemon.weight)?;
        table.raw_set("item", pokemon.item.as_deref())?;
        table.raw_set(
            "ability",
            if pokemon.ability_suppressed {
                None
            } else {
                pokemon.ability.as_deref()
            },
        )?;
        table.raw_set("happiness", pokemon.happiness)?;
        match pokemon.ivs {
            Some(ivs) => {
                let values = lua.create_table()?;
                for (key, value) in [
                    "hp",
                    "attack",
                    "defense",
                    "sp_attack",
                    "sp_defense",
                    "speed",
                ]
                .into_iter()
                .zip(ivs)
                {
                    values.raw_set(key, value)?;
                }
                table.raw_set("ivs", values)?;
            }
            None => table.raw_set("ivs", Value::Nil)?,
        }
        table.raw_set("side", side_name(id.side))?;
        table.raw_set("team", run.sides[id.side.index()].clone())?;
        table.raw_set("index", id.index + 1)?;
        table.raw_set("is_user", id == frame.actor)?;
        table.raw_set("is_ally", id.side == frame.actor.side && id != frame.actor)?;
        table.raw_set("is_foe", id.side != frame.actor.side)?;
        table.raw_set("fainted", pokemon.hp == 0)?;
        table.raw_set("confused", pokemon.confused_turns.is_some())?;
        table.raw_set("bound", pokemon.bound.is_some())?;
        table.raw_set("protected", pokemon.protected)?;
        table.raw_set("flinched", pokemon.flinched)?;
        table.raw_set("recharging", pokemon.recharging)?;
        table.raw_set(
            "hidden",
            match pokemon.hidden() {
                Some(kind) => symbol(lua, "Hidden", &format!("{kind:?}"))?,
                None => Value::Nil,
            },
        )?;
        table.raw_set("grounded", self.is_grounded(id))?;
        table.raw_set("trapped", self.is_trapped(id) || pokemon.bound.is_some())?;
        let (marks, effects) = self.conditions_tables(lua, Scope::Pokemon(id))?;
        table.raw_set("marks", marks)?;
        table.raw_set("effects", effects)?;
        table.raw_set("acted", pokemon.acted)?;
        let selected = self
            .turn_choices
            .iter()
            .find(|(own, _)| *own == id)
            .and_then(|(_, move_id)| self.catalog.get(move_id));
        match selected {
            Some(spec) => {
                let chosen = lua.create_table()?;
                chosen.raw_set("id", spec.id.as_str())?;
                chosen.raw_set("name", spec.name.as_str())?;
                chosen.raw_set(
                    "type",
                    symbol(lua, "Type", &format!("{:?}", spec.move_type))?,
                )?;
                chosen.raw_set(
                    "category",
                    symbol(lua, "Category", &format!("{:?}", spec.category))?,
                )?;
                chosen.raw_set("damaging", spec.category != crate::moves::Category::Status)?;
                chosen.raw_set("priority", spec.priority)?;
                table.raw_set("selected_move", chosen)?;
            }
            None => table.raw_set("selected_move", Value::Nil)?,
        }
        table.raw_set("damage_taken", pokemon.damage_taken)?;
        table.raw_set("hurt_this_turn", pokemon.hurt_this_turn)?;
        match &pokemon.last_hit {
            Some(hit) => {
                table.raw_set(
                    "last_attacker",
                    Self::pokemon_table(run, hit.attacker).cloned(),
                )?;
                table.raw_set("last_hit", self.hit_table(lua, hit)?)?;
            }
            None => {
                table.raw_set("last_attacker", Value::Nil)?;
                table.raw_set("last_hit", Value::Nil)?;
            }
        }
        table.raw_set(
            "turns_on_field",
            self.turn.saturating_sub(pokemon.entered_turn),
        )?;
        table.raw_set("last_move", pokemon.last_used.as_deref())?;
        table.raw_set("last_move_failed", pokemon.last_move_failed)?;
        table.raw_set(
            "streak",
            if pokemon.streak_move.as_deref() == Some(frame.spec.id.as_str()) {
                pokemon.streak
            } else {
                0
            },
        )?;
        let moves = lua.create_table()?;
        for own in &pokemon.moves {
            moves.raw_push(own.as_str())?;
        }
        table.raw_set("moves", moves)?;
        let used = lua.create_table()?;
        for own in &pokemon.moves_used {
            used.raw_set(own.as_str(), true)?;
        }
        table.raw_set("used_moves", used)?;
        let pp = lua.create_table()?;
        for (own, left) in &pokemon.move_pp {
            pp.raw_set(own.as_str(), *left)?;
        }
        table.raw_set("pp", pp)?;
        Ok(())
    }

    fn hit_table(&self, lua: &Lua, hit: &crate::model::HitInfo) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.raw_set("damage", hit.damage)?;
        table.raw_set("move", hit.move_id.as_str())?;
        table.raw_set(
            "category",
            symbol(lua, "Category", &format!("{:?}", hit.category))?,
        )?;
        table.raw_set(
            "type",
            symbol(lua, "Type", &format!("{:?}", hit.move_type))?,
        )?;
        table.raw_set("contact", hit.contact)?;
        Ok(table)
    }

    /// Everyone `frame`'s move is aimed at when a script does not name one Pokémon.
    fn recipients(&self, frame: &Frame) -> Vec<ParticipantId> {
        let actor = frame.actor;
        match frame.spec.target {
            Target::AllOthers => self
                .actors()
                .into_iter()
                .filter(|id| *id != actor)
                .collect(),
            Target::AllOpponents => self
                .actors()
                .into_iter()
                .filter(|id| id.side != actor.side)
                .collect(),
            Target::AllAllies => self
                .actors()
                .into_iter()
                .filter(|id| id.side == actor.side && *id != actor)
                .collect(),
            Target::UserAndAllies => self
                .actors()
                .into_iter()
                .filter(|id| id.side == actor.side)
                .collect(),
            Target::All => self.actors(),
            _ => vec![frame.target],
        }
    }

    /// Rewrites everything a script can read, so that it shows the battle as it is now.
    fn refresh(&self, run: &Run, frame: &Frame) -> mlua::Result<()> {
        let lua = self.catalog.lua();
        for (id, table) in &run.pokemon {
            self.fill_pokemon(lua, run, frame, *id, table)?;
        }
        let list = |ids: Vec<ParticipantId>| -> mlua::Result<Table> {
            let table = lua.create_table()?;
            for id in ids {
                table.raw_push(Self::pokemon_table(run, id).cloned())?;
            }
            Ok(table)
        };
        for side in Side::ALL {
            let table = &run.sides[side.index()];
            let (marks, effects) = self.conditions_tables(lua, Scope::Side(side))?;
            table.raw_set("marks", marks)?;
            table.raw_set("effects", effects)?;
            let last_faint = self.last_faint_turn[side.index()];
            table.raw_set("last_faint_turn", last_faint)?;
            table.raw_set(
                "fainted_last_turn",
                last_faint.is_some_and(|turn| turn + 1 == self.turn),
            )?;
            table.raw_set(
                "pokemon",
                list(
                    self.actors()
                        .into_iter()
                        .filter(|id| id.side == side)
                        .collect(),
                )?,
            )?;
        }
        let (marks, effects) = self.conditions_tables(lua, Scope::Field)?;
        run.field.raw_set("marks", marks)?;
        run.field.raw_set("effects", effects)?;
        let actor = frame.actor;
        let ctx = &run.ctx;
        ctx.raw_set(
            "allies",
            list(
                self.actors()
                    .into_iter()
                    .filter(|id| id.side == actor.side && *id != actor)
                    .collect(),
            )?,
        )?;
        ctx.raw_set(
            "foes",
            list(
                self.actors()
                    .into_iter()
                    .filter(|id| id.side != actor.side)
                    .collect(),
            )?,
        )?;
        ctx.raw_set(
            "others",
            list(
                self.actors()
                    .into_iter()
                    .filter(|id| *id != actor)
                    .collect(),
            )?,
        )?;
        ctx.raw_set("everyone", list(self.actors())?)?;
        ctx.raw_set("targets", list(self.recipients(frame))?)?;
        match &self.weather {
            Some(weather) => {
                ctx.raw_set(
                    "weather",
                    symbol(lua, "Weather", &format!("{:?}", weather.kind))?,
                )?;
                ctx.raw_set("weather_turns", weather.turns_left)?;
            }
            None => {
                ctx.raw_set("weather", Value::Nil)?;
                ctx.raw_set("weather_turns", Value::Nil)?;
            }
        }
        Ok(())
    }

    fn start_run(&self, frame: &Frame) -> Result<Run, BattleError> {
        let lua = self.catalog.lua();
        let handle = |scope: Scope| -> mlua::Result<Table> {
            let table = lua.create_table()?;
            table.raw_set("ref", lua.create_userdata(ScopeRef(scope))?)?;
            Ok(table)
        };
        let mut pokemon = Vec::new();
        let roster = lua.create_table()?;
        for side in Side::ALL {
            for index in 0..self.sides[side.index()].len() {
                let id = ParticipantId { side, index };
                let table = handle(Scope::Pokemon(id))?;
                roster.raw_push(table.clone())?;
                pokemon.push((id, table));
            }
        }
        let sides = [
            handle(Scope::Side(Side::Allies))?,
            handle(Scope::Side(Side::Foes))?,
        ];
        for side in Side::ALL {
            sides[side.index()].raw_set("name", side_name(side))?;
        }
        let field = handle(Scope::Field)?;
        let fields = lua.create_table()?;
        let spec = frame.spec;
        fields.raw_set("field", field.clone())?;
        fields.raw_set("user_side", sides[frame.actor.side.index()].clone())?;
        fields.raw_set(
            "foe_side",
            sides[frame.actor.side.opposite().index()].clone(),
        )?;
        for name in [
            "Status",
            "Stat",
            "Type",
            "TargetPolicy",
            "Weather",
            "Category",
            "Hidden",
            "Gender",
            "Rule",
        ] {
            fields.raw_set(name, lua.globals().get::<Value>(name)?)?;
        }
        let continuation = self.get(frame.actor).script_continuation.as_ref();
        fields.raw_set("turn", continuation.map_or(1, |lock| lock.turn))?;
        fields.raw_set("total_turns", continuation.map(|lock| lock.total_turns))?;
        fields.raw_set("battle_turn", self.turn)?;
        fields.raw_set("environment", self.environment.as_deref())?;
        let about = lua.create_table()?;
        about.raw_set("id", spec.id.as_str())?;
        about.raw_set("name", spec.name.as_str())?;
        about.raw_set(
            "type",
            symbol(lua, "Type", &format!("{:?}", spec.move_type))?,
        )?;
        about.raw_set(
            "category",
            symbol(lua, "Category", &format!("{:?}", spec.category))?,
        )?;
        about.raw_set("priority", spec.priority)?;
        let flags = lua.create_table()?;
        for flag in &spec.flags {
            flags.raw_set(flag.as_str(), true)?;
        }
        about.raw_set("flags", flags)?;
        fields.raw_set("move", about)?;
        let mut run = Run {
            thread: lua.create_thread(lua.create_function(|_, ()| Ok(()))?)?,
            ctx: fields.clone(),
            pokemon,
            sides,
            field,
        };
        fields.raw_set("user", Self::pokemon_table(&run, frame.actor).cloned())?;
        fields.raw_set("target", Self::pokemon_table(&run, frame.target).cloned())?;
        if frame.hook == ScriptHook::Hit
            && let Some(hit) = &self.get(frame.actor).last_hit
        {
            fields.raw_set("hit", self.hit_table(lua, hit)?)?;
        }
        self.refresh(&run, frame)?;
        let (thread, ctx) = self
            .catalog
            .start_script(spec, frame.hook, fields, roster)?;
        run.thread = thread;
        run.ctx = ctx;
        Ok(run)
    }

    /// Runs one hook of a scripted move. Returns whether the move landed a hit.
    pub(crate) fn execute_script(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
        events: &mut Vec<BattleEvent>,
        hook: ScriptHook,
    ) -> Result<bool, BattleError> {
        let mut frame = Frame {
            actor,
            target,
            spec,
            hook,
            any_hit: false,
            failed: false,
            checked: Vec::new(),
        };
        let run = self.start_run(&frame)?;
        let mut yielded = run.thread.resume::<Value>(run.ctx.clone())?;
        let mut operations = 0;
        while run.thread.status() == ThreadStatus::Resumable {
            operations += 1;
            if operations > MAX_OPERATIONS {
                return Err(BattleError::InvalidSetup(format!(
                    "script for {} exceeded {MAX_OPERATIONS} operations",
                    spec.id
                )));
            }
            let operation = match yielded {
                Value::UserData(ref userdata) => userdata.borrow::<ScriptOperation>()?.clone(),
                _ => {
                    return Err(BattleError::InvalidSetup(format!(
                        "script for {} yielded a value outside the battle API",
                        spec.id
                    )));
                }
            };
            if matches!(hook, ScriptHook::Hit | ScriptHook::TurnStart)
                && !reaction_may(&operation, hook)
            {
                return Err(BattleError::InvalidSetup(format!(
                    "{} hook for {} yielded an unsupported operation",
                    if hook == ScriptHook::Hit {
                        "hit"
                    } else {
                        "turn start"
                    },
                    spec.id
                )));
            }
            let response = match self.apply_operation(&mut frame, operation, events)? {
                Flow::Continue(value) => value,
                Flow::Stop => break,
            };
            self.refresh(&run, &frame)?;
            yielded = run.thread.resume::<Value>(response)?;
        }
        if hook == ScriptHook::Main {
            self.move_failed = frame.failed && !frame.any_hit;
        }
        Ok(frame.any_hit)
    }

    fn hit_reaches(
        &self,
        frame: &Frame,
        recipient: ParticipantId,
        options: &HitOptions,
        kind: HiddenKind,
    ) -> bool {
        options.hits_all_hidden
            || options.hits_hidden.contains(&kind)
            || frame.spec.hits_hidden.contains(&kind)
            || self.always_hits(frame.actor, recipient)
    }

    /// A hit that follows `try_hit` on the same Pokémon takes its answer instead of checking again.
    fn hit_outcome(
        &mut self,
        frame: &mut Frame,
        recipient: ParticipantId,
        options: &HitOptions,
        events: &mut Vec<BattleEvent>,
    ) -> HitOutcome {
        match frame.take_check(recipient) {
            Some(_) if self.get(recipient).hp == 0 => HitOutcome::Fainted,
            Some(true) => HitOutcome::Hit,
            Some(false) => HitOutcome::Missed,
            None => self.resolve_hit(frame, recipient, options, events),
        }
    }

    /// The checks every hit goes through: protection, hiding, then accuracy.
    fn resolve_hit(
        &mut self,
        frame: &Frame,
        recipient: ParticipantId,
        options: &HitOptions,
        events: &mut Vec<BattleEvent>,
    ) -> HitOutcome {
        let actor = frame.actor;
        if self.get(recipient).hp == 0 {
            return HitOutcome::Fainted;
        }
        if self.get(recipient).protected && recipient != actor && !options.ignore_protect {
            events.push(BattleEvent::Message(format!(
                "{} protected itself!",
                self.get(recipient).name
            )));
            return HitOutcome::Protected;
        }
        if let Some(kind) = self.get(recipient).hidden()
            && recipient != actor
            && !self.hit_reaches(frame, recipient, options, kind)
        {
            events.push(BattleEvent::Message(format!(
                "{} avoided the attack!",
                self.get(recipient).name
            )));
            return HitOutcome::Hidden;
        }
        let hit = if options.never_miss || self.always_hits(actor, recipient) {
            true
        } else if let Some(value) = options.accuracy {
            self.hit_check_chance(actor, recipient, value, options.ignore_evasion)
        } else {
            self.hit_check(actor, recipient, frame.spec, options.ignore_evasion)
        };
        if !hit {
            events.push(BattleEvent::Message(format!(
                "{}'s attack missed!",
                self.get(actor).name
            )));
            return HitOutcome::Missed;
        }
        HitOutcome::Hit
    }

    fn attack<'a>(&self, frame: &Frame<'a>, power: u16, options: &'a DamageOptions) -> Attack<'a> {
        Attack {
            power,
            high_crit: options.high_crit,
            always_crit: options.always_crit,
            consecutive_boost: 0,
            typeless: options.typeless,
            move_type: self.converted_type(
                frame.actor,
                options.move_type.unwrap_or(frame.spec.move_type),
            ),
            also_type: options.also_type,
            effective: &options.effective,
            category: options.category.unwrap_or(frame.spec.category),
            attack_from: options.attack_from.unwrap_or(frame.actor),
            attack_stat: options.attack_stat,
            defense_stat: options.defense_stat,
            ignore_stages: options.ignore_stages,
        }
    }

    fn op_damage(
        &mut self,
        frame: &mut Frame,
        power: u16,
        options: &DamageOptions,
        events: &mut Vec<BattleEvent>,
    ) -> Result<DamageReport, BattleError> {
        let actor = frame.actor;
        let targets = match options.target {
            Some(one) => vec![one],
            None => self.recipients(frame),
        };
        let mut report = DamageReport {
            effectiveness: 1.0,
            ..DamageReport::default()
        };
        for recipient in targets {
            match self.hit_outcome(frame, recipient, &options.hit, events) {
                HitOutcome::Hit => {}
                HitOutcome::Fainted => continue,
                HitOutcome::Protected => {
                    report.blocked = true;
                    report.unreached.push(recipient);
                    continue;
                }
                HitOutcome::Hidden | HitOutcome::Missed => {
                    report.missed = true;
                    report.unreached.push(recipient);
                    continue;
                }
            }
            let attack = self.attack(frame, power, options);
            let meta = HitMeta::of(frame.spec, attack.category, attack.move_type);
            let mut details = Vec::new();
            let Some(roll) = self.calculate(actor, recipient, &attack, &mut details) else {
                // Only the line saying that the hit does nothing.
                events.extend(details);
                report.immune = true;
                report.unreached.push(recipient);
                continue;
            };
            report.reached.push(recipient);
            let actual = roll
                .amount
                .min(self.get(recipient).hp.saturating_sub(options.min_target_hp));
            if actual > 0 {
                self.damage_from_move(actor, recipient, actual, &meta, events)?;
            }
            events.extend(details);
            report.hit = true;
            report.hits += 1;
            report.critical |= roll.critical;
            report.effectiveness = roll.effectiveness;
            report.fainted |= self.get(recipient).hp == 0;
            frame.any_hit = true;
            report.damage = report.damage.saturating_add(actual);
            if let Some(fraction) = options.recoil
                && actual > 0
            {
                let recoil = ((f32::from(actual) * fraction).floor() as u16).max(1);
                self.damage(actor, recoil, events);
                events.push(BattleEvent::Message(format!(
                    "{} was damaged by the recoil!",
                    self.get(actor).name
                )));
            }
            if let Some(fraction) = options.drain {
                if actual > 0 && self.get(actor).hp < self.get(actor).max_hp {
                    self.heal(
                        actor,
                        ((actual as f32 * fraction).floor() as u16).max(1),
                        events,
                    );
                }
                events.push(BattleEvent::Message(format!(
                    "{} had its energy drained!",
                    self.get(recipient).name
                )));
            }
        }
        if !report.hit {
            frame.failed = true;
        }
        Ok(report)
    }

    /// Several hits in a row on one Pokémon, as `Effect::MultiHit` does them.
    fn op_multi_hit(
        &mut self,
        frame: &mut Frame,
        power: u16,
        (min, max): (u8, u8),
        options: &DamageOptions,
        events: &mut Vec<BattleEvent>,
    ) -> Result<DamageReport, BattleError> {
        let actor = frame.actor;
        let recipient = options.target.unwrap_or(frame.target);
        let mut report = DamageReport {
            effectiveness: 1.0,
            ..DamageReport::default()
        };
        match self.hit_outcome(frame, recipient, &options.hit, events) {
            HitOutcome::Hit => {}
            HitOutcome::Protected => report.blocked = true,
            HitOutcome::Fainted | HitOutcome::Hidden | HitOutcome::Missed => report.missed = true,
        }
        if report.blocked || report.missed {
            frame.failed = true;
            report.unreached.push(recipient);
            return Ok(report);
        }
        let attack = self.attack(frame, power, options);
        let meta = HitMeta::of(frame.spec, attack.category, attack.move_type);
        if !attack.typeless && self.matchup(recipient, &attack) == 0.0 {
            self.immune_message(recipient, events);
            report.immune = true;
            frame.failed = true;
            report.unreached.push(recipient);
            return Ok(report);
        }
        report.reached.push(recipient);
        let hits = self.rng.range(min, max);
        // All hits are worked out against the target as it was before the first one.
        let mut amounts = Vec::new();
        let mut details = Vec::new();
        const CRIT: &str = "A critical hit!";
        let is_crit =
            |event: &BattleEvent| matches!(event, BattleEvent::Message(text) if text == CRIT);
        for _ in 0..hits {
            let mut hit_details = Vec::new();
            if let Some(roll) = self.calculate(actor, recipient, &attack, &mut hit_details) {
                amounts.push(roll.amount);
                report.critical |= roll.critical;
                report.effectiveness = roll.effectiveness;
                if details.is_empty() {
                    details.extend(hit_details.iter().filter(|event| !is_crit(event)).cloned());
                }
                if hit_details.iter().any(is_crit) && !details.iter().any(is_crit) {
                    details.push(BattleEvent::Message(CRIT.into()));
                }
            }
        }
        for amount in amounts {
            let actual = amount.min(self.get(recipient).hp.saturating_sub(options.min_target_hp));
            if actual > 0 {
                self.damage_from_move(actor, recipient, actual, &meta, events)?;
            }
            report.damage = report.damage.saturating_add(actual);
        }
        events.extend(details);
        events.push(BattleEvent::Message(format!("Hit {hits} times!")));
        report.hit = true;
        report.hits = hits;
        report.fainted = self.get(recipient).hp == 0;
        frame.any_hit = true;
        Ok(report)
    }

    fn report_table(&self, report: &DamageReport) -> mlua::Result<Value> {
        let table = self.catalog.lua().create_table()?;
        table.raw_set("hit", report.hit)?;
        table.raw_set("damage", report.damage)?;
        table.raw_set("fainted", report.fainted)?;
        table.raw_set("critical", report.critical)?;
        table.raw_set("effectiveness", report.effectiveness)?;
        table.raw_set("missed", report.missed)?;
        table.raw_set("blocked", report.blocked)?;
        table.raw_set("immune", report.immune)?;
        table.raw_set("hits", report.hits)?;
        // Positions in the roster the script API was given, for `ctx:reached`.
        let allies = self.sides[Side::Allies.index()].len();
        let positions = |ids: &[ParticipantId]| -> Vec<usize> {
            ids.iter()
                .map(|id| match id.side {
                    Side::Allies => id.index + 1,
                    Side::Foes => allies + id.index + 1,
                })
                .collect()
        };
        table.raw_set("reached", positions(&report.reached))?;
        table.raw_set("unreached", positions(&report.unreached))?;
        Ok(Value::Table(table))
    }

    /// A chance of an extra effect: always certain at 1, otherwise rolled.
    fn effect_roll(&mut self, actor: ParticipantId, chance: f32) -> bool {
        chance >= 1.0 || {
            let chance = self.effect_chance(actor, chance);
            self.rng.chance(chance)
        }
    }

    fn hp_points(&self, who: ParticipantId, amount: HpAmount, round_up: bool) -> u16 {
        match amount {
            HpAmount::Points(points) => points,
            HpAmount::MaxFraction(fraction) => {
                let share = f32::from(self.get(who).max_hp) * fraction;
                (if round_up {
                    share.ceil()
                } else {
                    share.floor()
                } as u16)
                    .max(1)
            }
        }
    }

    /// Gives `who` a status the way `Effect::Status` does. Returns whether it took.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn inflict_status(
        &mut self,
        actor: ParticipantId,
        who: ParticipantId,
        status: AppliedStatus,
        replace: bool,
        move_type: crate::model::PokemonType,
        announce_failure: bool,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let new_status = status.battle_status();
        let current = self.get(who).status;
        let name = self.get(who).name.clone();
        let immune = type_chart::status_immune(new_status, move_type, &self.types_of(who))
            || (who != actor && self.blocks_status(who, new_status));
        if self.get(who).hp > 0 && (current.is_none() || replace) && !immune {
            let turns = match status {
                AppliedStatus::RestSleep => 2,
                AppliedStatus::Asleep => self.rng.range(1, 3),
                AppliedStatus::BadlyPoisoned => 1,
                _ => 0,
            };
            let pokemon = self.get_mut(who);
            pokemon.status = Some(new_status);
            pokemon.status_turns = turns;
            events.push(BattleEvent::Status {
                target: who,
                status: Some(new_status),
            });
            events.push(BattleEvent::Message(match status {
                AppliedStatus::RestSleep => format!("{name} slept and became healthy!"),
                AppliedStatus::Asleep => format!("{name} fell asleep!"),
                AppliedStatus::Frozen => format!("{name} was frozen solid!"),
                _ => format!("{name} was {}!", status.description()),
            }));
            return true;
        }
        if announce_failure {
            events.push(BattleEvent::Message(if immune {
                format!("It doesn't affect {name}...")
            } else if current == Some(new_status) {
                format!("{name} was already {}!", status.description())
            } else {
                "But it failed!".into()
            }));
        }
        false
    }

    fn cure_status(
        &mut self,
        who: ParticipantId,
        only: Option<Status>,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let current = self.get(who).status;
        if current.is_none() || only.is_some_and(|status| current != Some(status)) {
            return false;
        }
        let pokemon = self.get_mut(who);
        pokemon.status = None;
        pokemon.status_turns = 0;
        events.push(BattleEvent::Status {
            target: who,
            status: None,
        });
        true
    }

    fn confuse(
        &mut self,
        actor: ParticipantId,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if self.get(who).hp == 0
            || self.get(who).confused_turns.is_some()
            || (who != actor && self.blocks_confusion(who))
        {
            return false;
        }
        let turns = self.rng.range(1, 4);
        self.get_mut(who).confused_turns = Some(turns);
        events.push(BattleEvent::Message(format!(
            "{} became confused!",
            self.get(who).name
        )));
        true
    }

    fn apply_operation(
        &mut self,
        frame: &mut Frame,
        operation: ScriptOperation,
        events: &mut Vec<BattleEvent>,
    ) -> Result<Flow, BattleError> {
        let actor = frame.actor;
        let target = frame.target;
        let spec = frame.spec;
        let value = match operation {
            ScriptOperation::WatchHitsUntilNextAction => {
                if spec.on_hit.is_none() {
                    return Err(BattleError::InvalidSetup(format!(
                        "move {} has no hit hook",
                        spec.id
                    )));
                }
                self.hit_reactions
                    .retain(|reaction| reaction.owner != actor || reaction.move_id != spec.id);
                self.hit_reactions.push(HitReaction {
                    owner: actor,
                    move_id: spec.id.clone(),
                });
                Value::Nil
            }
            ScriptOperation::HealSelf(fraction) => {
                let pokemon = self.get(actor);
                let amount = (pokemon.max_hp as f32 * fraction).ceil() as u16;
                if pokemon.hp < pokemon.max_hp {
                    self.heal(actor, amount, events);
                    events.push(BattleEvent::Message(format!(
                        "{} regained health!",
                        self.get(actor).name
                    )));
                } else {
                    events.push(BattleEvent::Message("But it failed!".into()));
                    frame.failed = true;
                }
                Value::Nil
            }
            ScriptOperation::FlinchTarget(chance) => {
                let chance = self.effect_chance(actor, chance);
                if self.get(target).hp > 0 && self.rng.chance(chance) {
                    self.get_mut(target).flinched = true;
                }
                Value::Nil
            }
            ScriptOperation::ForceMove {
                total_turns,
                target_policy,
            } => {
                if self.get(actor).script_continuation.is_some() {
                    return Err(BattleError::InvalidSetup("move is already forced".into()));
                }
                self.get_mut(actor).script_continuation = Some(ScriptContinuation {
                    move_id: spec.id.clone(),
                    target,
                    turn: 1,
                    total_turns,
                    target_policy,
                });
                Value::Nil
            }
            ScriptOperation::BreakSequence => {
                self.get_mut(actor).script_continuation = None;
                Value::Nil
            }
            ScriptOperation::RandomInt { min, max } => {
                Value::Integer(self.rng.range(min, max) as i64)
            }
            ScriptOperation::Message(message) => {
                events.push(BattleEvent::Message(message));
                Value::Nil
            }
            ScriptOperation::Announce => {
                events.push(BattleEvent::Message(format!(
                    "{} used {}!",
                    self.get(actor).name,
                    spec.name
                )));
                Value::Nil
            }
            ScriptOperation::ConfuseSelf => {
                if self.get(actor).hp > 0 && self.get(actor).confused_turns.is_none() {
                    let turns = self.rng.range(1, 4);
                    self.get_mut(actor).confused_turns = Some(turns);
                    events.push(BattleEvent::Message(format!(
                        "{} became confused!",
                        self.get(actor).name
                    )));
                }
                Value::Nil
            }
            ScriptOperation::RecoilMaxHp(fraction) => {
                let max_hp = self.get(actor).max_hp;
                let amount = ((f32::from(max_hp) * fraction).round() as u16).max(1);
                self.damage(actor, amount, events);
                events.push(BattleEvent::Message(format!(
                    "{} was damaged by the recoil!",
                    self.get(actor).name
                )));
                Value::Nil
            }
            ScriptOperation::ChangeSelfStat { stat, stages } => {
                self.change_stat(actor, stat, stages, events);
                Value::Nil
            }
            ScriptOperation::BoostNextMove {
                move_type,
                multiplier,
            } => {
                self.get_mut(actor).next_move_power_boost = Some(NextMovePowerBoost {
                    move_type,
                    multiplier,
                });
                Value::Nil
            }
            ScriptOperation::Fail => {
                events.push(BattleEvent::Message("But it failed!".into()));
                frame.failed = true;
                return Ok(Flow::Stop);
            }
            ScriptOperation::FaintUser => {
                let remaining = self.get(actor).hp;
                self.damage(actor, remaining, events);
                Value::Nil
            }
            ScriptOperation::Recharge => {
                if self.get(actor).hp > 0 {
                    self.get_mut(actor).recharging = true;
                }
                Value::Nil
            }
            ScriptOperation::Damage { power, options } => {
                let report = self.op_damage(frame, power, &options, events)?;
                self.report_table(&report)?
            }
            ScriptOperation::MultiHit {
                power,
                min,
                max,
                options,
            } => {
                let report = self.op_multi_hit(frame, power, (min, max), &options, events)?;
                self.report_table(&report)?
            }
            ScriptOperation::DirectDamage {
                who,
                amount,
                typeless,
                min_target_hp,
            } => {
                let mut report = DamageReport {
                    effectiveness: 1.0,
                    ..DamageReport::default()
                };
                if self.get(who).hp == 0 {
                    report.missed = true;
                } else if !typeless && self.effectiveness(who, spec.move_type) == 0.0 {
                    self.immune_message(who, events);
                    report.immune = true;
                    report.unreached.push(who);
                } else {
                    report.reached.push(who);
                    let actual = amount.min(self.get(who).hp.saturating_sub(min_target_hp));
                    if actual > 0 {
                        let meta = HitMeta::of(spec, spec.category, spec.move_type);
                        self.damage_from_move(actor, who, actual, &meta, events)?;
                    }
                    report.hit = true;
                    report.hits = 1;
                    report.damage = actual;
                    report.fainted = self.get(who).hp == 0;
                    frame.any_hit = true;
                }
                if !report.hit {
                    frame.failed = true;
                }
                self.report_table(&report)?
            }
            ScriptOperation::TryHit { who, options } => {
                let recipient = who.unwrap_or(target);
                let hit = self.resolve_hit(frame, recipient, &options, events) == HitOutcome::Hit;
                if !hit {
                    frame.failed = true;
                }
                frame.checked.retain(|(own, _)| *own != recipient);
                frame.checked.push((recipient, hit));
                Value::Boolean(hit)
            }
            ScriptOperation::Chance(probability) => Value::Boolean(self.rng.chance(probability)),
            ScriptOperation::EffectChance(probability) => {
                Value::Boolean(self.effect_roll(actor, probability))
            }
            ScriptOperation::Hurt { who, amount } => {
                let points = self.hp_points(who, amount, false).min(self.get(who).hp);
                if points > 0 {
                    self.damage(who, points, events);
                }
                Value::Integer(i64::from(points))
            }
            ScriptOperation::Heal { who, amount } => {
                let pokemon = self.get(who);
                let missing = pokemon.max_hp - pokemon.hp;
                let points = if pokemon.hp == 0 {
                    0
                } else {
                    self.hp_points(who, amount, true).min(missing)
                };
                if points > 0 {
                    self.heal(who, points, events);
                }
                Value::Integer(i64::from(points))
            }
            ScriptOperation::ApplyStatus {
                who,
                status,
                chance,
                replace,
                announce_failure,
            } => Value::Boolean(
                self.effect_roll(actor, chance)
                    && self.inflict_status(
                        actor,
                        who,
                        status,
                        replace,
                        spec.move_type,
                        announce_failure,
                        events,
                    ),
            ),
            ScriptOperation::CureStatus { who, status } => Value::Boolean(self.cure_status(
                who,
                status.map(AppliedStatus::battle_status),
                events,
            )),
            ScriptOperation::Confuse { who, chance } => {
                Value::Boolean(self.effect_roll(actor, chance) && self.confuse(actor, who, events))
            }
            ScriptOperation::Flinch { who, chance } => {
                let flinched = self.get(who).hp > 0 && self.effect_roll(actor, chance);
                if flinched {
                    self.get_mut(who).flinched = true;
                }
                Value::Boolean(flinched)
            }
            ScriptOperation::ChangeStat {
                who,
                stat,
                stages,
                chance,
            } => {
                let change = if self.get(who).hp == 0 || !self.effect_roll(actor, chance) {
                    0
                } else if stages < 0 && who != actor && self.blocks_stat_drops(who) {
                    events.push(BattleEvent::Message(format!(
                        "{}'s stats were not lowered!",
                        self.get(who).name
                    )));
                    0
                } else {
                    self.change_stat(who, stat, stages, events)
                };
                Value::Integer(i64::from(change))
            }
            ScriptOperation::ResetStats { who } => {
                let stages: Vec<(Stat, i8)> = self
                    .get(who)
                    .stages
                    .iter()
                    .filter(|(_, stage)| **stage != 0)
                    .map(|(stat, stage)| (*stat, *stage))
                    .collect();
                self.get_mut(who).stages.clear();
                for (stat, stage) in &stages {
                    events.push(BattleEvent::StatChange {
                        target: who,
                        stat: *stat,
                        stages: -stage,
                    });
                }
                Value::Boolean(!stages.is_empty())
            }
            ScriptOperation::Bind {
                who,
                min_turns,
                max_turns,
            } => {
                let bound = self.get(who).hp > 0
                    && self.effectiveness(who, spec.move_type) != 0.0
                    && self.get(who).bound.is_none();
                if bound {
                    let turns = self.rng.range(min_turns, max_turns);
                    self.get_mut(who).bound = Some(Bound {
                        turns_left: turns,
                        source: spec.name.clone(),
                    });
                    events.push(BattleEvent::Message(format!(
                        "{} was squeezed by {}!",
                        self.get(who).name,
                        self.get(actor).name
                    )));
                }
                Value::Boolean(bound)
            }
            ScriptOperation::Free { who } => {
                Value::Boolean(self.get_mut(who).bound.take().is_some())
            }
            ScriptOperation::Protect { who } => {
                // As with the Protect effect, a Pokémon that keeps protecting itself with the
                // same move succeeds a third as often for each turn in a row it has worked.
                let user = self.get(actor);
                let streak = if who == actor && user.streak_move.as_deref() == Some(&spec.id) {
                    user.streak
                } else {
                    0
                };
                let holds = streak == 0 || self.rng.chance((1.0f32 / 3.0).powi(i32::from(streak)));
                if !holds {
                    events.push(BattleEvent::Message("But it failed!".into()));
                    frame.failed = true;
                }
                let protect = holds && !self.get(who).protected && self.get(who).hp > 0;
                if protect {
                    self.get_mut(who).protected = true;
                    events.push(BattleEvent::Message(format!(
                        "{} protected itself!",
                        self.get(who).name
                    )));
                }
                Value::Boolean(protect)
            }
            ScriptOperation::BreakProtect { who } => {
                Value::Boolean(std::mem::take(&mut self.get_mut(who).protected))
            }
            ScriptOperation::Hide(kind) => {
                let pokemon = self.get_mut(actor);
                pokemon.semi_invulnerable = true;
                pokemon.hidden_kind = kind;
                Value::Nil
            }
            ScriptOperation::Unhide { who } => {
                let pokemon = self.get_mut(who);
                let was_hidden = std::mem::take(&mut pokemon.semi_invulnerable);
                if was_hidden {
                    // Whatever it was in the middle of is called off.
                    pokemon.script_continuation = None;
                    if pokemon
                        .locked_move
                        .as_ref()
                        .is_some_and(|lock| lock.charging)
                    {
                        pokemon.locked_move = None;
                    }
                }
                Value::Boolean(was_hidden)
            }
            ScriptOperation::SetWeather { kind, turns } => {
                Value::Boolean(self.start_weather(kind, turns, events))
            }
            ScriptOperation::ClearWeather => {
                let cleared = self.weather.take().is_some();
                if cleared {
                    events.push(BattleEvent::Weather(None));
                }
                Value::Boolean(cleared)
            }
            ScriptOperation::Mark {
                scope,
                name,
                value,
                turns,
            } => {
                self.set_mark(scope, &name, value, turns);
                Value::Nil
            }
            ScriptOperation::Unmark { scope, name }
            | ScriptOperation::EndEffect { scope, name } => {
                Value::Boolean(self.end_condition(scope, &name))
            }
            ScriptOperation::StartEffect {
                scope,
                condition,
                restart,
            } => Value::Boolean(self.start_condition(scope, condition, restart)),
            ScriptOperation::EndGroup { scope, group } => {
                Value::Boolean(self.end_group(scope, &group))
            }
            ScriptOperation::SetTypes { who, types } => {
                self.get_mut(who).types = types;
                Value::Nil
            }
            ScriptOperation::SetWeight { who, weight } => {
                self.get_mut(who).weight = weight;
                Value::Nil
            }
            ScriptOperation::TakeItem { who } => match self.get_mut(who).item.take() {
                Some(item) => Value::String(self.catalog.lua().create_string(&item)?),
                None => Value::Nil,
            },
            ScriptOperation::GiveItem { who, item } => {
                let free = self.get(who).item.is_none() && self.get(who).hp > 0;
                if free {
                    self.get_mut(who).item = Some(item);
                }
                Value::Boolean(free)
            }
            ScriptOperation::SuppressAbility { who } => {
                let pokemon = self.get_mut(who);
                let had = pokemon.ability.is_some() && !pokemon.ability_suppressed;
                pokemon.ability_suppressed = true;
                Value::Boolean(had)
            }
            ScriptOperation::AddPayout(amount) => {
                let payout = &mut self.payout[actor.side.index()];
                *payout = payout.saturating_add(amount);
                Value::Nil
            }
            ScriptOperation::Hurry => {
                self.hurry = Some(spec.id.clone());
                Value::Nil
            }
        };
        Ok(Flow::Continue(value))
    }

    /// Sets the weather as `Effect::Weather` does. Returns false if it was already that.
    pub(crate) fn start_weather(
        &mut self,
        kind: crate::model::WeatherKind,
        turns: u8,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if self
            .weather
            .as_ref()
            .is_some_and(|weather| weather.kind == kind)
        {
            events.push(BattleEvent::Message("But it failed!".into()));
            return false;
        }
        let weather = Weather {
            kind,
            turns_left: turns,
        };
        self.weather = Some(weather.clone());
        events.push(BattleEvent::Message(kind.messages().0.into()));
        events.push(BattleEvent::Weather(Some(weather)));
        true
    }
}

/// What a hook that runs in the middle of someone else's action may do.
fn reaction_may(operation: &ScriptOperation, hook: ScriptHook) -> bool {
    use ScriptOperation as Op;
    match operation {
        Op::Message(_)
        | Op::ChangeSelfStat { .. }
        | Op::ChangeStat { .. }
        | Op::ResetStats { .. }
        | Op::ApplyStatus { .. }
        | Op::CureStatus { .. }
        | Op::Confuse { .. }
        | Op::Flinch { .. }
        | Op::Hurt { .. }
        | Op::Heal { .. }
        | Op::Mark { .. }
        | Op::Unmark { .. }
        | Op::StartEffect { .. }
        | Op::EndEffect { .. }
        | Op::EndGroup { .. }
        | Op::Chance(_)
        | Op::EffectChance(_)
        | Op::RandomInt { .. }
        | Op::TakeItem { .. }
        | Op::GiveItem { .. }
        | Op::AddPayout(_) => true,
        Op::WatchHitsUntilNextAction | Op::Protect { .. } => hook == ScriptHook::TurnStart,
        _ => false,
    }
}
