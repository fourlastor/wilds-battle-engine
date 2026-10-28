//! Ways a wild battle ends without a winner: fleeing foes, blown-away Pokémon, Teleport.
use wilds_battle_engine::{
    ActionSelection, AdvanceResult, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice,
    MoveCatalog, Pokemon, Side,
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
fn mon(name: &str, moves: &[&str], speed: u16) -> Pokemon {
    let mut p = Pokemon::new(name, moves.iter().map(|m| m.to_string()).collect());
    p.hp = 300;
    p.max_hp = 300;
    p.speed = speed;
    p
}
fn battle(allies: Vec<Pokemon>, foes: Vec<Pokemon>, wild: bool) -> Battle {
    let mut b = Battle::with_rng(Fixed, [allies, foes], MoveCatalog::builtin().unwrap()).unwrap();
    if b.participants(Side::Allies).len() > 1 {
        b.set_active([vec![0], vec![0]]).unwrap();
    }
    b.set_wild_battle(wild).unwrap();
    b
}
/// One turn: the ally answers with `ally`, the foe with `foe` (a move ID or "escape").
fn turn(b: &mut Battle, ally: &str, foe: &str) -> AdvanceResult {
    loop {
        let result = b.advance().unwrap();
        let AdvanceStatus::Awaiting(p) = &result.status else {
            return result;
        };
        if b.turn() > 0 && !result.events.is_empty() {
            return result;
        }
        let wanted = if p.actor.side == Side::Allies {
            ally
        } else {
            foe
        };
        let choice = p
            .choices
            .iter()
            .find(|c| match c {
                Choice::Escape { .. } => wanted == "escape",
                Choice::UseMove { move_id, .. } => move_id == wanted,
                _ => false,
            })
            .unwrap_or_else(|| panic!("{wanted} not in {p:?}"));
        b.set_response(ActionSelection {
            prompt_id: p.id,
            choice_id: choice.id(),
        })
        .unwrap();
    }
}
fn fled(result: &AdvanceResult, side: Side) -> bool {
    result.status == AdvanceStatus::Escaped { side }
}

#[test]
fn wild_foes_can_run_away() {
    let mut b = battle(
        vec![mon("Ally", &["splash"], 50)],
        vec![mon("Foe", &["splash"], 100)],
        true,
    );
    let result = turn(&mut b, "splash", "escape");
    assert!(fled(&result, Side::Foes), "{result:?}");

    // A trapped foe cannot.
    let mut b = battle(
        vec![mon("Ally", &["mean-look"], 200)],
        vec![mon("Foe", &["splash"], 100)],
        true,
    );
    turn(&mut b, "mean-look", "splash");
    let AdvanceStatus::Awaiting(p) = b.advance().unwrap().status else {
        panic!()
    };
    assert_eq!(p.actor.side, Side::Allies);
    b.set_response(ActionSelection {
        prompt_id: p.id,
        choice_id: 0,
    })
    .unwrap();
    let AdvanceStatus::Awaiting(p) = b.advance().unwrap().status else {
        panic!()
    };
    assert!(!p.choices.iter().any(|c| matches!(c, Choice::Escape { .. })));
}

#[test]
fn blowing_away_a_lone_wild_pokemon_ends_the_battle() {
    for blow in ["roar", "whirlwind", "dragon-tail", "circle-throw"] {
        let mut b = battle(
            vec![mon("Ally", &[blow], 50)],
            vec![mon("Foe", &["splash"], 100)],
            true,
        );
        let result = turn(&mut b, blow, "splash");
        assert!(fled(&result, Side::Foes), "{blow}: {result:?}");
        assert!(
            result
                .events
                .iter()
                .any(|e| matches!(e, BattleEvent::Message(m) if m == "Foe fled!"))
        );
    }
    // The wild Pokémon's Roar sends the player's Pokémon away too.
    let mut b = battle(
        vec![mon("Ally", &["splash"], 50)],
        vec![mon("Foe", &["roar"], 100)],
        true,
    );
    assert!(fled(&turn(&mut b, "splash", "roar"), Side::Allies));
    // A trainer battle without a replacement: the move fails, the battle goes on.
    let mut b = battle(
        vec![mon("Ally", &["roar"], 50)],
        vec![mon("Foe", &["splash"], 100)],
        false,
    );
    assert!(matches!(
        turn(&mut b, "roar", "splash").status,
        AdvanceStatus::Awaiting(_)
    ));
    // With a bench, it switches as usual.
    let mut b = battle(
        vec![mon("Ally", &["splash"], 50), mon("Bench", &["splash"], 50)],
        vec![mon("Foe", &["roar"], 100)],
        true,
    );
    let result = turn(&mut b, "splash", "roar");
    assert!(matches!(result.status, AdvanceStatus::Awaiting(_)));
    assert_eq!(b.active_indices(Side::Allies), [1]);
}

#[test]
fn teleport_leaves_wild_battles_and_switches_in_trainer_battles() {
    let mut b = battle(
        vec![mon("Ally", &["splash"], 50)],
        vec![mon("Foe", &["teleport"], 100)],
        true,
    );
    assert!(fled(&turn(&mut b, "splash", "teleport"), Side::Foes));

    let mut b = battle(
        vec![
            mon("Ally", &["teleport"], 50),
            mon("Bench", &["splash"], 50),
        ],
        vec![mon("Foe", &["splash"], 100)],
        false,
    );
    let result = turn(&mut b, "teleport", "splash");
    let AdvanceStatus::Awaiting(p) = result.status else {
        panic!()
    };
    assert_eq!(p.reason, wilds_battle_engine::PromptReason::Pivot);
}
