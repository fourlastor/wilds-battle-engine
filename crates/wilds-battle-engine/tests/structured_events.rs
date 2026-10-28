//! Events that time animations: moves used, hits, misses, charging and hiding.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, HiddenKind,
    MoveCatalog, ParticipantId, Pokemon, PokemonType, Side, Stat,
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
    p.hp = 500;
    p.max_hp = 500;
    p
}
fn battle(allies: Vec<Pokemon>, foes: Vec<Pokemon>) -> Battle {
    Battle::with_rng(Fixed, [allies, foes], MoveCatalog::builtin().unwrap()).unwrap()
}
/// One turn where each prompt picks `moves[actor]`, or splash.
fn turn(b: &mut Battle, ally_move: &str) -> Vec<BattleEvent> {
    let mut events = Vec::new();
    loop {
        let result = b.advance().unwrap();
        events.extend(result.events);
        let AdvanceStatus::Awaiting(prompt) = result.status else {
            return events;
        };
        if !events.is_empty() && prompt.actor.side == Side::Allies && prompt.actor.index == 0 {
            return events;
        }
        let wanted = if prompt.actor.side == Side::Allies {
            ally_move
        } else {
            "splash"
        };
        let choice = prompt
            .choices
            .iter()
            .find(|c| matches!(c, Choice::UseMove { move_id, .. } if move_id == wanted))
            .or_else(|| prompt.choices.first())
            .unwrap();
        b.set_response(ActionSelection {
            prompt_id: prompt.id,
            choice_id: choice.id(),
        })
        .unwrap();
    }
}
fn position(events: &[BattleEvent], f: impl Fn(&BattleEvent) -> bool) -> usize {
    events
        .iter()
        .position(f)
        .unwrap_or_else(|| panic!("{events:?}"))
}

#[test]
fn a_move_is_announced_then_hits_before_the_damage() {
    let mut b = battle(
        vec![mon("Ally", &["tackle"])],
        vec![mon("Foe", &["splash"])],
    );
    let events = turn(&mut b, "tackle");
    let used = position(&events, |e| {
        matches!(e, BattleEvent::MoveUsed { actor, move_id, targets }
            if *actor == id(Side::Allies, 0) && move_id == "tackle" && *targets == [id(Side::Foes, 0)])
    });
    assert!(matches!(&events[used + 1], BattleEvent::Message(m) if m == "Ally used Tackle!"));
    let hit = position(&events, |e| {
        matches!(
            e,
            BattleEvent::Hit {
                hit: 1,
                effectiveness_percent: 100,
                critical: false,
                ..
            }
        )
    });
    assert!(
        matches!(&events[hit + 1], BattleEvent::Damage { .. }),
        "{events:?}"
    );
}

#[test]
fn hits_are_counted_and_report_effectiveness() {
    let mut foe = mon("Foe", &["splash"]);
    foe.types = vec![PokemonType::Normal];
    let mut b = battle(vec![mon("Ally", &["double-kick"])], vec![foe]);
    let events = turn(&mut b, "double-kick");
    let hits: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            BattleEvent::Hit {
                hit,
                effectiveness_percent,
                ..
            } => Some((*hit, *effectiveness_percent)),
            _ => None,
        })
        .collect();
    assert_eq!(hits, [(1, 200), (2, 200)]);
}

#[test]
fn misses_name_the_attacker_and_target() {
    let mut foe = mon("Foe", &["splash"]);
    foe.stages.insert(Stat::Evasion, 6);
    let mut b = battle(vec![mon("Ally", &["tackle"])], vec![foe]);
    let events = turn(&mut b, "tackle");
    position(&events, |e| {
        *e == BattleEvent::Miss {
            actor: id(Side::Allies, 0),
            target: id(Side::Foes, 0),
        }
    });
}

#[test]
fn two_turn_moves_charge_hide_and_reappear() {
    let mut b = battle(vec![mon("Ally", &["fly"])], vec![mon("Foe", &["splash"])]);
    let first = turn(&mut b, "fly");
    let charging = position(
        &first,
        |e| matches!(e, BattleEvent::Charging { move_id, .. } if move_id == "fly"),
    );
    let hidden = position(&first, |e| {
        *e == BattleEvent::Hidden {
            target: id(Side::Allies, 0),
            place: Some(HiddenKind::Air),
        }
    });
    assert!(charging < hidden);
    let second = turn(&mut b, "fly");
    let used = position(&second, |e| matches!(e, BattleEvent::MoveUsed { .. }));
    let shown = position(&second, |e| {
        *e == BattleEvent::Hidden {
            target: id(Side::Allies, 0),
            place: None,
        }
    });
    let hit = position(&second, |e| matches!(e, BattleEvent::Hit { .. }));
    assert!(used < shown && shown < hit, "{second:?}");

    let mut b = battle(
        vec![mon("Ally", &["solar-beam"])],
        vec![mon("Foe", &["splash"])],
    );
    let first = turn(&mut b, "solar-beam");
    position(&first, |e| matches!(e, BattleEvent::Charging { .. }));
    assert!(
        !first
            .iter()
            .any(|e| matches!(e, BattleEvent::Hidden { .. }))
    );
}

#[test]
fn participation_lists_the_opponents_each_pokemon_faced() {
    let mut b = battle(
        vec![mon("Lead", &["splash"]), mon("Bench", &["splash"])],
        vec![mon("Foe", &["splash"])],
    );
    b.set_active([vec![0], vec![0]]).unwrap();
    turn(&mut b, "splash");
    assert_eq!(b.faced_by(id(Side::Allies, 0)), [0]);
    assert!(b.faced_by(id(Side::Allies, 1)).is_empty());
    assert_eq!(b.faced_by(id(Side::Foes, 0)), [0]);
}

#[test]
fn moves_start_with_the_hosts_pp_and_maximum() {
    let mut ally = mon("Ally", &["tackle", "growl"]);
    ally.pp_max.insert("tackle".into(), 56);
    ally.move_pp.insert("growl".into(), 3);
    let b = battle(vec![ally], vec![mon("Foe", &["splash"])]);
    let ally = id(Side::Allies, 0);
    assert_eq!(b.max_pp(ally, "tackle"), Some(56));
    assert_eq!(b.participants(Side::Allies)[0].move_pp["tackle"], 56);
    assert_eq!(b.max_pp(ally, "growl"), Some(40));
    assert_eq!(b.participants(Side::Allies)[0].move_pp["growl"], 3);
}

#[test]
fn field_moves_aim_at_no_one() {
    let mut b = battle(
        vec![mon("Ally", &["sunny-day"])],
        vec![mon("Foe", &["splash"])],
    );
    let events = turn(&mut b, "sunny-day");
    position(&events, |e| {
        matches!(e, BattleEvent::MoveUsed { move_id, targets, .. }
            if move_id == "sunny-day" && targets.is_empty())
    });
}
