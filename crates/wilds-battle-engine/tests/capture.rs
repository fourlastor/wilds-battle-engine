//! Throwing balls at wild Pokémon.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, ContextValue,
    MoveCatalog, ParticipantId, Pokemon, PokemonType, Prompt, Side, Status,
};

/// Every draw returns `self.0`, so `index(n)` is `self.0 % n`.
struct Draw(u64);
impl BattleRng for Draw {
    fn next_u64(&mut self) -> u64 {
        self.0
    }
    fn fraction(&mut self) -> f32 {
        0.5
    }
}
fn id(side: Side, index: usize) -> ParticipantId {
    ParticipantId { side, index }
}
fn mon(name: &str) -> Pokemon {
    let mut p = Pokemon::new(name, vec!["splash".into()]);
    p.hp = 150;
    p.max_hp = 150;
    p.level = 50;
    p
}
fn wild(draw: u64, ally: Pokemon, foe: Pokemon, bag: &[&str]) -> Battle {
    let mut b = Battle::with_rng(
        Draw(draw),
        [vec![ally], vec![foe]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_wild_battle(true).unwrap();
    b.set_bag(Side::Allies, bag.iter().map(|i| i.to_string()).collect())
        .unwrap();
    b
}
fn catchable(rate: u8) -> Pokemon {
    let mut foe = mon("Foe");
    foe.catch_rate = Some(rate);
    foe
}
fn prompt(b: &mut Battle) -> Prompt {
    match b.advance().unwrap().status {
        AdvanceStatus::Awaiting(prompt) => prompt,
        other => panic!("no prompt: {other:?}"),
    }
}
/// Throws `ball`, lets the foe splash, and returns the turn's result.
fn throw(b: &mut Battle, ball: &str) -> wilds_battle_engine::AdvanceResult {
    let p = prompt(b);
    let choice = p
        .choices
        .iter()
        .find(|c| matches!(c, Choice::UseItem { item, .. } if item == ball))
        .unwrap_or_else(|| panic!("{p:?}"));
    b.set_response(ActionSelection {
        prompt_id: p.id,
        choice_id: choice.id(),
    })
    .unwrap();
    let p = prompt(b);
    b.set_response(ActionSelection {
        prompt_id: p.id,
        choice_id: 0,
    })
    .unwrap();
    b.advance().unwrap()
}
fn capture(events: &[BattleEvent]) -> (u8, bool, bool) {
    events
        .iter()
        .find_map(|e| match e {
            BattleEvent::Capture {
                shakes,
                critical,
                caught,
                ..
            } => Some((*shakes, *critical, *caught)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{events:?}"))
}
fn said(events: &[BattleEvent], text: &str) -> bool {
    events
        .iter()
        .any(|e| matches!(e, BattleEvent::Message(m) if m == text))
}

#[test]
fn a_ball_that_holds_four_checks_catches_and_ends_the_battle() {
    let mut b = wild(0, mon("Ally"), catchable(45), &["poke-ball"]);
    let result = throw(&mut b, "poke-ball");
    assert_eq!(capture(&result.events), (4, false, true));
    assert!(said(&result.events, "Gotcha! Foe was caught!"));
    let target = id(Side::Foes, 0);
    assert_eq!(result.status, AdvanceStatus::Captured { target });
    assert_eq!(
        b.advance().unwrap().status,
        AdvanceStatus::Captured { target }
    );
}

#[test]
fn ball_modifiers_change_the_odds() {
    // Catch rate 45 at full HP: a Poké Ball holds below 38528, a Net Ball (3.5x) below 48724.
    let mut b = wild(
        45000,
        mon("Ally"),
        catchable(45),
        &["poke-ball", "net-ball"],
    );
    let result = throw(&mut b, "poke-ball");
    assert_eq!(capture(&result.events), (0, false, false));
    assert!(said(&result.events, "Oh no! The Pokémon broke free!"));
    assert!(matches!(result.status, AdvanceStatus::Awaiting(_)));

    let mut bug = catchable(45);
    bug.types = vec![PokemonType::Bug];
    let mut b = wild(45000, mon("Ally"), bug, &["net-ball"]);
    assert!(capture(&throw(&mut b, "net-ball").events).2);
}

#[test]
fn low_hp_status_and_guaranteed_balls_catch_regardless_of_the_roll() {
    let mut weak = catchable(255);
    weak.hp = 1;
    weak.status = Some(Status::Asleep);
    let mut b = wild(u64::MAX, mon("Ally"), weak, &["poke-ball"]);
    assert!(capture(&throw(&mut b, "poke-ball").events).2);

    let mut b = wild(u64::MAX, mon("Ally"), catchable(3), &["master-ball"]);
    assert!(capture(&throw(&mut b, "master-ball").events).2);
}

#[test]
fn critical_captures_need_one_check() {
    let mut b = wild(0, mon("Ally"), catchable(45), &["poke-ball"]);
    b.set_context("critical_capture", ContextValue::Number(2.5));
    assert_eq!(capture(&throw(&mut b, "poke-ball").events), (1, true, true));
}

#[test]
fn uncatchable_pokemon_refuse_the_ball_and_keep_it_in_the_bag() {
    let mut foe = catchable(255);
    foe.uncatchable = true;
    let mut b = wild(0, mon("Ally"), foe, &["poke-ball"]);
    let result = throw(&mut b, "poke-ball");
    assert!(
        result
            .events
            .iter()
            .any(|e| matches!(e, BattleEvent::CaptureRefused { .. }))
    );
    assert!(
        !result
            .events
            .iter()
            .any(|e| matches!(e, BattleEvent::Capture { .. }))
    );
    assert!(result.events.iter().any(|e| matches!(
        e,
        BattleEvent::ItemUsed {
            consumed: false,
            ..
        }
    )));
    assert!(matches!(result.status, AdvanceStatus::Awaiting(_)));
}

#[test]
fn balls_are_for_wild_battles() {
    let mut b = Battle::with_rng(
        Draw(0),
        [vec![mon("Ally")], vec![catchable(45)]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    b.set_bag(Side::Allies, vec!["poke-ball".into()]).unwrap();
    let p = prompt(&mut b);
    assert!(
        !p.choices
            .iter()
            .any(|c| matches!(c, Choice::UseItem { .. }))
    );
}

#[test]
fn ball_fetch_picks_up_the_first_failed_ball() {
    let mut fetcher = mon("Ally");
    fetcher.ability = Some("ball-fetch".into());
    let mut b = wild(45000, fetcher, catchable(45), &["poke-ball", "great-ball"]);
    let result = throw(&mut b, "great-ball");
    assert!(!capture(&result.events).2);
    assert!(
        said(&result.events, "Ally found a great ball!"),
        "{:?}",
        result.events
    );
    assert_eq!(
        b.participants(Side::Allies)[0].item.as_deref(),
        Some("great-ball")
    );
}

#[test]
fn ball_scripts_read_host_facts() {
    // Quick Ball: 5x on the first turn only.
    let mut b = wild(45000, mon("Ally"), catchable(45), &["quick-ball"]);
    assert!(capture(&throw(&mut b, "quick-ball").events).2);
    // Repeat Ball: 3.5x for species the host flags as registered.
    let mut registered = catchable(45);
    registered.flags.insert("registered".into());
    let mut b = wild(45000, mon("Ally"), registered, &["repeat-ball"]);
    assert!(capture(&throw(&mut b, "repeat-ball").events).2);
    let mut b = wild(45000, mon("Ally"), catchable(45), &["repeat-ball"]);
    assert!(!capture(&throw(&mut b, "repeat-ball").events).2);
    // Dusk Ball: 3x at night.
    let mut b = wild(45000, mon("Ally"), catchable(45), &["dusk-ball"]);
    b.set_context("time_of_day", ContextValue::Text("night".into()));
    assert!(capture(&throw(&mut b, "dusk-ball").events).2);
}
