//! A host's Lua script for one battle: hooks, queued actions, faint prevention and weather.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, MessageArg,
    MoveCatalog, ParticipantId, Pokemon, PokemonType, Side, Status, WeatherKind,
};

struct Fixed;
impl BattleRng for Fixed {
    fn next_u64(&mut self) -> u64 {
        0
    }
    fn fraction(&mut self) -> f32 {
        0.5
    }
}
fn id(side: Side, index: usize) -> ParticipantId {
    ParticipantId { side, index }
}
fn mon(name: &str, moves: &[&str]) -> Pokemon {
    let mut p = Pokemon::new(name, moves.iter().map(|m| m.to_string()).collect());
    p.hp = 400;
    p.max_hp = 400;
    p
}
fn battle(ally: Pokemon, foe: Pokemon, script: &str) -> Battle {
    let mut b = Battle::with_rng(
        Fixed,
        [vec![ally], vec![foe]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_script(script).unwrap();
    b
}
/// Plays turns with every prompt answered by its first move, until `turns` have passed or
/// the battle ends; returns the events.
fn play(b: &mut Battle, turns: u64) -> Vec<BattleEvent> {
    let mut events = Vec::new();
    loop {
        let result = b.advance().unwrap();
        events.extend(result.events);
        let AdvanceStatus::Awaiting(p) = result.status else {
            return events;
        };
        if b.turn() >= turns && p.actor.side == Side::Allies {
            return events;
        }
        let choice = p
            .choices
            .iter()
            .find(|c| matches!(c, Choice::UseMove { .. }))
            .unwrap();
        b.set_response(ActionSelection {
            prompt_id: p.id,
            choice_id: choice.id(),
        })
        .unwrap();
    }
}
fn scripted(events: &[BattleEvent], key: &str) -> Vec<Vec<MessageArg>> {
    events
        .iter()
        .filter_map(|e| match e {
            BattleEvent::Script { key: k, args } if k == key => Some(args.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_foe_gets_an_extra_action_every_third_action() {
    let script = r#"
        local actions = 0
        return {
          moves = {
            {id = "tremor", name = "Tremor", type = Type.Ground, category = Category.Physical,
             pp = 1, script = function(ctx) ctx:direct_damage(ctx.target, 30, {scaled = true}) end},
          },
          after_action = function(ctx)
            if ctx.user.side ~= "foes" then return end
            actions = actions + 1
            if actions % 3 == 0 then ctx:queue_action(ctx.user, "tremor") end
          end,
        }"#;
    let mut ally = mon("Ally", &["splash"]);
    ally.types = vec![PokemonType::Electric];
    let mut b = battle(ally, mon("Foe", &["splash"]), script);
    let events = play(&mut b, 6);
    let tremors: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, BattleEvent::MoveUsed { move_id, .. } if move_id == "tremor"))
        .collect();
    assert_eq!(tremors.len(), 2, "after the third and sixth foe actions");
    // Fixed damage, doubled against an Electric type.
    assert_eq!(b.participants(Side::Allies)[0].hp, 400 - 2 * 60);
}

#[test]
fn a_multi_phase_foe_survives_its_first_knockout_and_becomes_catchable() {
    let script = r#"
        local phase = 1
        return {
          on_start = function(ctx) ctx:set_catchable(ctx.user, false) end,
          would_faint = function(ctx)
            if ctx.user.side ~= "foes" or phase > 1 then return end
            phase = 2
            ctx:prevent_faint(ctx.user.max_hp)
            ctx:set_catchable(ctx.user, true)
            ctx:set_flag(ctx.user, "second_phase")
            ctx:emit("phase", ctx.user, phase)
            ctx:message("{0} is not finished yet!", ctx.user)
          end,
        }"#;
    let mut foe = mon("Foe", &["splash"]);
    foe.hp = 1;
    foe.catch_rate = Some(45);
    let mut ally = mon("Ally", &["tackle"]);
    ally.attack = 2000;
    let mut b = battle(ally, foe, script);
    let events = play(&mut b, 1);
    let foe = &b.participants(Side::Foes)[0];
    assert_eq!(foe.hp, 400, "{events:?}");
    assert!(!foe.uncatchable && foe.flags.contains("second_phase"));
    assert!(!events.iter().any(|e| matches!(e, BattleEvent::Fainted(_))));
    assert_eq!(
        scripted(&events, "phase"),
        [vec![
            MessageArg::Pokemon {
                id: id(Side::Foes, 0),
                name: "Foe".into(),
                gender: Default::default()
            },
            MessageArg::Number(2)
        ]]
    );

    // The second knockout is final.
    let events = play(&mut b, 3);
    assert!(
        events
            .iter()
            .any(|e| *e == BattleEvent::Fainted(id(Side::Foes, 0))),
        "{events:?}"
    );
}

#[test]
fn a_timed_weather_pauses_while_its_holder_sleeps_and_reports_its_end() {
    let script = r#"
        local holder
        return {
          on_start = function(ctx)
            holder = ctx.user
            ctx:set_weather(Weather.Rain, 2)
          end,
          weather_tick = function(ctx)
            for _, pokemon in ipairs(ctx.everyone) do
              if pokemon.name == holder.name
                 and (pokemon.status == Status.Asleep or pokemon.status == Status.Frozen) then
                ctx:hold_weather()
              end
            end
          end,
          on_weather_end = function(ctx) ctx:emit("weather_over", ctx.detail) end,
        }"#;
    let mut sleeper = mon("Foe", &["splash"]);
    sleeper.status = Some(Status::Asleep);
    sleeper.status_turns = 50;
    let mut b = battle(mon("Ally", &["splash"]), sleeper, script);
    let events = play(&mut b, 6);
    assert_eq!(b.weather().map(|w| w.kind), Some(WeatherKind::Rain));
    assert!(scripted(&events, "weather_over").is_empty());

    let mut b = battle(mon("Ally", &["splash"]), mon("Foe", &["splash"]), script);
    let events = play(&mut b, 6);
    assert!(b.weather().is_none());
    assert_eq!(
        scripted(&events, "weather_over"),
        [vec![MessageArg::Text("Rain".into())]]
    );
}

#[test]
fn turn_hooks_set_rules_flags_and_endless_weather() {
    let script = r#"
        return {
          on_turn_start = function(ctx)
            if ctx.battle_turn == 2 then ctx:set_no_escape(true) end
          end,
          on_turn_end = function(ctx)
            ctx:emit("turn_over", ctx.battle_turn)
            if ctx.battle_turn == 1 then
              ctx:set_weather(Weather.Sandstorm)
              ctx:set_terrain("Misty", 3)
            end
          end,
        }"#;
    let mut b = battle(mon("Ally", &["splash"]), mon("Foe", &["splash"]), script);
    b.set_wild_battle(true).unwrap();
    let events = play(&mut b, 5);
    assert_eq!(
        scripted(&events, "turn_over")
            .into_iter()
            .map(|args| args[0].clone())
            .collect::<Vec<_>>(),
        (1..=5).map(MessageArg::Number).collect::<Vec<_>>()
    );
    assert!(b.no_escape());
    assert_eq!(b.weather().map(|w| w.kind), Some(WeatherKind::Sandstorm));
    assert!(
        !b.field_conditions()
            .iter()
            .any(|c| c.name == "misty_terrain"),
        "3 turns passed"
    );
}

#[test]
fn script_errors_are_reported() {
    let mut b = battle(
        mon("Ally", &["splash"]),
        mon("Foe", &["splash"]),
        "return {on_start = function(ctx) error('broken hook') end}",
    );
    assert!(b.advance().unwrap_err().to_string().contains("broken hook"));

    let mut b = Battle::new(
        1,
        [
            vec![mon("Ally", &["splash"])],
            vec![mon("Foe", &["splash"])],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    assert!(
        b.set_script("return {on_nothing = function() end}")
            .is_err()
    );
    assert!(
        b.set_script("return {on_start = function(ctx) local r = math.random end}")
            .is_ok()
    );
}
