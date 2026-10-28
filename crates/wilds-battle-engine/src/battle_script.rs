//! A host's Lua script for one battle: hooks that run at set moments, and moves of its own.
use std::collections::{HashMap, VecDeque};

use crate::engine::Battle;
use crate::model::{BattleError, BattleEvent, ParticipantId, Side};
use crate::moves::{MoveSpec, ScriptHook};

/// The moments a battle script can act at, by their Lua names.
pub(crate) const HOOKS: [&str; 7] = [
    "on_start",
    "on_turn_start",
    "on_turn_end",
    "after_action",
    "would_faint",
    "weather_tick",
    "on_weather_end",
];

#[derive(Clone, Debug, Default)]
pub(crate) struct BattleScript {
    /// Each hook as a scripted move whose `script` is the hook function.
    pub(crate) hooks: HashMap<&'static str, MoveSpec>,
    /// Moves the script defines for `ctx:queue_action`, by ID.
    pub(crate) moves: HashMap<String, MoveSpec>,
}

/// What hooks asked for while they ran.
#[derive(Debug, Default)]
pub(crate) struct ScriptState {
    /// Extra actions to take once the current action ends: who, move, target.
    pub(crate) queued: VecDeque<(ParticipantId, String, Option<ParticipantId>)>,
    /// `ctx:prevent_faint(hp)` from `would_faint`.
    pub(crate) prevented_faint: Option<u16>,
    /// `ctx:hold_weather()` from `weather_tick`.
    pub(crate) weather_held: bool,
    /// The running hook's name and detail, for `ctx.hook` and `ctx.detail`.
    pub(crate) hook: Option<(&'static str, Option<String>)>,
    /// An error from a hook that ran where errors cannot be returned, reported by `advance`.
    pub(crate) error: Option<BattleError>,
}

impl Battle {
    /// Installs the battle's script; see "Battle scripts" in the README. Must be called before
    /// the first `advance`.
    pub fn set_script(&mut self, source: &str) -> Result<(), BattleError> {
        if self.has_started() {
            return Err(BattleError::InvalidSetup(
                "a battle script must be set before the battle starts".into(),
            ));
        }
        self.script = Some(self.catalog.battle_script_from_lua(source)?);
        Ok(())
    }

    pub(crate) fn has_hook(&self, name: &str) -> bool {
        self.script
            .as_ref()
            .is_some_and(|script| script.hooks.contains_key(name))
    }

    /// The Pokémon a hook without one of its own runs for: the first living active foe, else
    /// the first living active ally.
    fn default_subject(&self) -> Option<ParticipantId> {
        let actors = self.actors();
        actors
            .iter()
            .copied()
            .find(|id| id.side == Side::Foes)
            .or_else(|| actors.first().copied())
    }

    /// Runs hook `name` with `subject` as `ctx.user`, if the script has it. Errors are kept
    /// for `advance` to report, since some hooks run deep inside other actions.
    pub(crate) fn hook(
        &mut self,
        name: &'static str,
        subject: Option<ParticipantId>,
        detail: Option<String>,
        events: &mut Vec<BattleEvent>,
    ) {
        let Some(spec) = self
            .script
            .as_ref()
            .and_then(|script| script.hooks.get(name))
            .cloned()
        else {
            return;
        };
        let Some(subject) = subject.or_else(|| self.default_subject()) else {
            return;
        };
        let outer = self.script_state.hook.replace((name, detail));
        let result = self.execute_script(subject, subject, &spec, events, ScriptHook::Main);
        self.script_state.hook = outer;
        if let Err(error) = result {
            self.script_state.error.get_or_insert(error);
        }
    }

    /// Takes the actions hooks queued, in order.
    pub(crate) fn run_queued_actions(
        &mut self,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        while let Some((who, move_id, target)) = self.script_state.queued.pop_front() {
            if self.winner.is_some() || self.get(who).hp == 0 || !self.is_active(who) {
                continue;
            }
            let target = target
                .filter(|target| self.get(*target).hp > 0)
                .or_else(|| self.actors().into_iter().find(|id| id.side != who.side))
                .unwrap_or(who);
            let own = self
                .script
                .as_ref()
                .and_then(|script| script.moves.get(&move_id))
                .cloned();
            match own {
                Some(spec) => {
                    self.announce(who, target, &spec, events);
                    self.execute_script(who, target, &spec, events, ScriptHook::Main)?;
                }
                None => {
                    self.call_move(who, &move_id, Some(target), events)?;
                }
            }
            self.resolve_outcome(events);
            self.sync_hidden(events);
        }
        Ok(())
    }

    /// `would_faint`: lets the script keep `target` in the battle with the HP it chooses.
    /// Returns the HP to restore, if it did.
    pub(crate) fn faint_prevented(
        &mut self,
        target: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> Option<u16> {
        if !self.has_hook("would_faint")
            || matches!(self.script_state.hook, Some(("would_faint", _)))
        {
            return None;
        }
        self.script_state.prevented_faint = None;
        self.hook("would_faint", Some(target), None, events);
        self.script_state
            .prevented_faint
            .take()
            .map(|hp| hp.clamp(1, self.get(target).max_hp))
    }

    /// Whether `weather_tick` held the weather's countdown this turn.
    pub(crate) fn weather_tick_held(&mut self, events: &mut Vec<BattleEvent>) -> bool {
        self.script_state.weather_held = false;
        self.hook("weather_tick", None, None, events);
        std::mem::take(&mut self.script_state.weather_held)
    }
}
