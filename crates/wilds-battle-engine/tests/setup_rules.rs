//! Weather, terrain, conditions, flags and surroundings a battle starts with.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, ContextValue,
    MoveCatalog, Pokemon, Side, Stat, TerrainKind, WeatherKind,
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

fn mon(name: &str, moves: &[&str]) -> Pokemon {
    let mut p = Pokemon::new(name, moves.iter().map(|m| m.to_string()).collect());
    p.hp = 500;
    p.max_hp = 500;
    p
}

fn battle_with(catalog: MoveCatalog, ally: Pokemon, foe: Pokemon) -> Battle {
    Battle::with_rng(Fixed, [vec![ally], vec![foe]], catalog).unwrap()
}

fn battle(ally: &[&str], foe: &[&str]) -> Battle {
    battle_with(
        MoveCatalog::builtin().unwrap(),
        mon("Ally", ally),
        mon("Foe", foe),
    )
}

/// Answers every prompt with its first move choice for `turns` turns; returns all events.
fn play(battle: &mut Battle, turns: u64) -> Vec<BattleEvent> {
    let mut events = Vec::new();
    while battle.turn() < turns {
        let result = battle.advance().unwrap();
        events.extend(result.events);
        let AdvanceStatus::Awaiting(prompt) = result.status else {
            break;
        };
        let choice = prompt
            .choices
            .iter()
            .find(|c| matches!(c, Choice::UseMove { .. }))
            .unwrap();
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: choice.id(),
            })
            .unwrap();
    }
    events.extend(battle.advance().unwrap().events);
    events
}

fn said(events: &[BattleEvent], text: &str) -> bool {
    events
        .iter()
        .any(|e| matches!(e, BattleEvent::Message(m) if m == text))
}

#[test]
fn starting_weather_without_turns_lasts_until_replaced() {
    let mut b = battle(&["splash"], &["splash"]);
    b.set_starting_weather(WeatherKind::Rain, None).unwrap();
    let events = play(&mut b, 8);
    assert!(said(&events, "It started to rain!"), "{events:?}");
    assert!(!said(&events, "The rain stopped."), "{events:?}");
    assert_eq!(b.weather().unwrap().kind, WeatherKind::Rain);

    let mut b = battle(&["splash"], &["splash"]);
    b.set_starting_weather(WeatherKind::Sandstorm, Some(2))
        .unwrap();
    let events = play(&mut b, 4);
    assert!(said(&events, "The sandstorm subsided."), "{events:?}");
    assert!(b.weather().is_none());
}

#[test]
fn starting_weather_is_replaced_by_moves_and_set_before_entry_abilities() {
    let mut drizzle = mon("Ally", &["splash"]);
    drizzle.ability = Some("drizzle".into());
    let mut b = battle_with(
        MoveCatalog::builtin().unwrap(),
        drizzle,
        mon("Foe", &["splash"]),
    );
    b.set_starting_weather(WeatherKind::Sun, None).unwrap();
    play(&mut b, 1);
    assert_eq!(b.weather().unwrap().kind, WeatherKind::Rain);

    let mut b = battle(&["sunny-day"], &["splash"]);
    b.set_starting_weather(WeatherKind::Hail, None).unwrap();
    play(&mut b, 1);
    assert_eq!(b.weather().unwrap().kind, WeatherKind::Sun);
}

#[test]
fn starting_terrain_is_the_terrain_moves_and_abilities_set() {
    let mut b = battle(&["spore"], &["splash"]);
    b.set_starting_terrain(TerrainKind::Electric, Some(5))
        .unwrap();
    let events = play(&mut b, 1);
    assert!(
        said(
            &events,
            "An electric current is running across the battlefield!"
        ),
        "{events:?}"
    );
    let terrain = &b.field_conditions()[0];
    assert_eq!(terrain.name, "electric_terrain");
    assert_eq!(terrain.group.as_deref(), Some("terrain"));
    // Grounded Pokémon cannot fall asleep on Electric Terrain.
    assert!(b.participants(Side::Foes)[0].status.is_none(), "{events:?}");
}

#[test]
fn starting_conditions_come_from_a_lua_list() {
    let mut b = battle(&["splash"], &["splash"]);
    b.start_conditions_lua(
        r#"return {
            {scope = "allies", name = "safeguard", turns = 3, rules = {{kind = Rule.BlockStatus}}},
            {scope = "field", name = "mark", value = 7},
            {pokemon = {side = "foes", index = 0}, name = "focus", end_message = "{0} relaxed."}
        }"#,
    )
    .unwrap();
    assert_eq!(b.side_conditions(Side::Allies)[0].name, "safeguard");
    assert_eq!(b.side_conditions(Side::Allies)[0].turns_left, Some(3));
    assert_eq!(b.field_conditions()[0].value, 7);
    assert_eq!(b.participants(Side::Foes)[0].conditions[0].name, "focus");

    let error = battle(&["splash"], &["splash"])
        .start_conditions_lua(r#"return {{pokemon = {side = "foes", index = 4}, name = "x"}}"#)
        .unwrap_err();
    assert!(error.to_string().contains("missing participant"), "{error}");
    let error = battle(&["splash"], &["splash"])
        .start_conditions_lua(r#"return {{scope = "moon", name = "x"}}"#)
        .unwrap_err();
    assert!(error.to_string().contains("allies"), "{error}");

    let mut started = battle(&["splash"], &["splash"]);
    started.advance().unwrap();
    assert!(started.start_conditions_lua("return {}").is_err());
    assert!(
        started
            .set_starting_weather(WeatherKind::Rain, None)
            .is_err()
    );
}

#[test]
fn no_escape_removes_running_from_wild_battles() {
    let mut b = battle(&["splash"], &["splash"]);
    b.set_wild_battle(true).unwrap();
    b.set_no_escape(true);
    let AdvanceStatus::Awaiting(prompt) = b.advance().unwrap().status else {
        panic!()
    };
    assert!(
        !prompt
            .choices
            .iter()
            .any(|c| matches!(c, Choice::Escape { .. }))
    );
}

#[test]
fn participants_start_with_stages_confusion_and_catch_data() {
    let mut ally = mon("Ally", &["tackle"]);
    ally.stages.insert(Stat::Attack, 2);
    ally.confused_turns = Some(3);
    let mut foe = mon("Foe", &["splash"]);
    foe.uncatchable = true;
    foe.catch_rate = Some(45);
    let mut b = battle_with(MoveCatalog::builtin().unwrap(), ally, foe);
    let events = play(&mut b, 1);
    assert!(said(&events, "Ally is confused!"), "{events:?}");
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), 2);
    assert!(b.participants(Side::Foes)[0].uncatchable);
}

#[test]
fn scripts_read_the_battle_context() {
    let catalog = MoveCatalog::from_lua(
        r#"return {
        {id = "look", name = "Look", type = Type.Normal, category = Category.Status, pp = 5,
         target = Target.User, script = function(ctx)
           ctx:message("{0} sees the " .. ctx.context.time_of_day .. " sky.", ctx.user)
           if ctx.context.indoors == false and ctx.context.depth == 3 then ctx:message("Outdoors.") end
           if ctx.user.uncatchable then ctx:message("Uncatchable!") end
         end}
    }"#,
    )
    .unwrap();
    let mut b = battle_with(catalog, mon("Ally", &["look"]), mon("Foe", &["look"]));
    b.set_context("time_of_day", ContextValue::Text("night".into()));
    b.set_context("indoors", ContextValue::Flag(false));
    b.set_context("depth", ContextValue::Number(3.0));
    let events = play(&mut b, 1);
    assert!(said(&events, "Ally sees the night sky."), "{events:?}");
    assert!(said(&events, "Outdoors."), "{events:?}");
    assert!(!said(&events, "Uncatchable!"), "{events:?}");
}
