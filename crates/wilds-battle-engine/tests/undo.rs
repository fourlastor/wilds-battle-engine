//! Taking back a choice made earlier in the same turn.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, MoveCatalog,
    ParticipantId, Pokemon, Prompt, PromptReason, Side,
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
    let mut p = Pokemon::new(name, moves.iter().map(|s| s.to_string()).collect());
    p.hp = 1000;
    p.max_hp = 1000;
    p
}
fn game(allies: Vec<Pokemon>, foes: Vec<Pokemon>, active: Option<[Vec<usize>; 2]>) -> Battle {
    let mut b = Battle::with_rng(Fixed, [allies, foes], MoveCatalog::builtin().unwrap()).unwrap();
    if let Some(active) = active {
        b.set_active(active).unwrap();
    }
    b
}
fn prompt(b: &mut Battle) -> Prompt {
    match b.advance().unwrap().status {
        AdvanceStatus::Awaiting(prompt) => prompt,
        other => panic!("no prompt: {other:?}"),
    }
}
fn answer(b: &mut Battle, p: &Prompt, f: impl Fn(&Choice) -> bool) {
    let choice = p.choices.iter().find(|c| f(c)).expect("choice");
    b.set_response(ActionSelection {
        prompt_id: p.id,
        choice_id: choice.id(),
    })
    .unwrap();
}
fn uses(name: &str) -> impl Fn(&Choice) -> bool + '_ {
    move |c| matches!(c, Choice::UseMove { move_id, .. } if move_id == name)
}
fn is_undo(c: &Choice) -> bool {
    matches!(c, Choice::Undo { .. })
}

#[test]
fn undo_returns_to_the_previous_ally_and_replaces_its_choice() {
    let mut b = game(
        vec![
            mon("Ally A", &["tackle", "growl"]),
            mon("Ally B", &["splash"]),
        ],
        vec![mon("Foe A", &["splash"]), mon("Foe B", &["splash"])],
        None,
    );
    let first = prompt(&mut b);
    assert_eq!(first.actor, id(Side::Allies, 0));
    assert!(!first.choices.iter().any(is_undo));
    answer(&mut b, &first, uses("tackle"));

    let second = prompt(&mut b);
    assert_eq!(second.actor, id(Side::Allies, 1));
    assert!(
        is_undo(second.choices.last().unwrap()),
        "undo is listed last"
    );
    assert_eq!(
        second.choices.iter().filter(|c| is_undo(c)).count(),
        1,
        "{second:?}"
    );
    answer(&mut b, &second, is_undo);

    let again = prompt(&mut b);
    assert_eq!(again.actor, id(Side::Allies, 0));
    assert!(!again.choices.iter().any(is_undo));
    answer(&mut b, &again, uses("growl"));
    let second = prompt(&mut b);
    answer(&mut b, &second, uses("splash"));

    // Foes are offered undo too once one of them chose; a host can ignore it.
    let foe_a = prompt(&mut b);
    assert_eq!(foe_a.actor, id(Side::Foes, 0));
    assert!(
        !foe_a.choices.iter().any(is_undo),
        "allies' choices are not the foes' to undo"
    );
    answer(&mut b, &foe_a, uses("splash"));
    let foe_b = prompt(&mut b);
    assert!(foe_b.choices.iter().any(is_undo));
    answer(&mut b, &foe_b, uses("splash"));

    let events = b.advance().unwrap().events;
    let said = |text: &str| {
        events
            .iter()
            .any(|e| matches!(e, BattleEvent::Message(m) if m == text))
    };
    assert!(said("Ally A used Growl!"), "{events:?}");
    assert!(!said("Ally A used Tackle!"), "{events:?}");
}

#[test]
fn undo_releases_a_reserved_switch_target() {
    let mut b = game(
        vec![
            mon("Ally A", &["splash"]),
            mon("Ally B", &["splash"]),
            mon("Bench", &["splash"]),
        ],
        vec![mon("Foe A", &["splash"]), mon("Foe B", &["splash"])],
        Some([vec![0, 1], vec![0, 1]]),
    );
    let switches_to_bench = |p: &Prompt| {
        p.choices
            .iter()
            .any(|c| matches!(c, Choice::Switch { to, .. } if to.index == 2))
    };
    let first = prompt(&mut b);
    answer(&mut b, &first, |c| matches!(c, Choice::Switch { .. }));
    let second = prompt(&mut b);
    assert!(!switches_to_bench(&second), "the bench member is reserved");
    answer(&mut b, &second, is_undo);
    let again = prompt(&mut b);
    assert_eq!(again.actor, id(Side::Allies, 0));
    assert!(switches_to_bench(&again));
}

#[test]
fn singles_and_replacements_never_offer_undo() {
    let mut b = game(
        vec![mon("Ally", &["splash"]), mon("Bench", &["splash"])],
        vec![mon("Foe", &["splash"])],
        Some([vec![0], vec![0]]),
    );
    for _ in 0..2 {
        let p = prompt(&mut b);
        assert!(!p.choices.iter().any(is_undo), "{p:?}");
        answer(&mut b, &p, uses("splash"));
    }

    let mut fainted = mon("Fainted", &["splash"]);
    fainted.hp = 0;
    let mut b = game(
        vec![fainted, mon("Reserve", &["splash"])],
        vec![mon("Foe", &["splash"])],
        Some([vec![0], vec![0]]),
    );
    let p = prompt(&mut b);
    assert_eq!(p.reason, PromptReason::Replacement);
    assert!(!p.choices.iter().any(is_undo));
}
