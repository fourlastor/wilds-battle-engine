//! Items used from the bag as a turn action.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, MoveCatalog, MoveFile,
    ParticipantId, Pokemon, PokemonAppearance, Prompt, Side, Stat, Status,
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
fn battle_with(catalog: MoveCatalog, allies: Vec<Pokemon>, foes: Vec<Pokemon>) -> Battle {
    let mut b = Battle::with_rng(Fixed, [allies, foes], catalog).unwrap();
    if b.participants(Side::Allies).len() > 1 {
        b.set_active([vec![0], vec![0]]).unwrap();
    }
    b
}
fn battle(allies: Vec<Pokemon>, foes: Vec<Pokemon>) -> Battle {
    battle_with(MoveCatalog::builtin().unwrap(), allies, foes)
}
fn prompt(b: &mut Battle) -> (Prompt, Vec<BattleEvent>) {
    let result = b.advance().unwrap();
    match result.status {
        AdvanceStatus::Awaiting(prompt) => (prompt, result.events),
        other => panic!("no prompt: {other:?}"),
    }
}
fn items(p: &Prompt) -> Vec<(String, ParticipantId, Option<String>)> {
    p.choices
        .iter()
        .filter_map(|c| match c {
            Choice::UseItem {
                item,
                target,
                move_id,
                ..
            } => Some((item.clone(), *target, move_id.clone())),
            _ => None,
        })
        .collect()
}
fn answer(b: &mut Battle, p: &Prompt, f: impl Fn(&Choice) -> bool) {
    let choice = p
        .choices
        .iter()
        .find(|c| f(c))
        .unwrap_or_else(|| panic!("{p:?}"));
    b.set_response(ActionSelection {
        prompt_id: p.id,
        choice_id: choice.id(),
    })
    .unwrap();
}
fn use_item(item: &str, on: ParticipantId) -> impl Fn(&Choice) -> bool + '_ {
    move |c| matches!(c, Choice::UseItem { item: i, target, move_id: None, .. } if i == item && *target == on)
}
fn uses(name: &str) -> impl Fn(&Choice) -> bool + '_ {
    move |c| matches!(c, Choice::UseMove { move_id, .. } if move_id == name)
}
/// The ally uses `item` on `on`, the foe answers with its first move; returns the turn's events.
fn turn_with_item(b: &mut Battle, item: &str, on: ParticipantId) -> Vec<BattleEvent> {
    let (p, _) = prompt(b);
    answer(b, &p, use_item(item, on));
    let (p, _) = prompt(b);
    answer(b, &p, |c| matches!(c, Choice::UseMove { .. }));
    b.advance().unwrap().events
}
fn said(events: &[BattleEvent], text: &str) -> bool {
    events
        .iter()
        .any(|e| matches!(e, BattleEvent::Message(m) if m == text))
}

#[test]
fn every_builtin_item_loads() {
    let catalog = MoveCatalog::builtin().unwrap();
    let ids: Vec<_> = MoveCatalog::builtin_item_ids().collect();
    assert!(ids.len() >= 50, "{ids:?}");
    for id in ids {
        assert_eq!(catalog.load_item(id).unwrap().expect(id).id, id);
    }
}

#[test]
fn a_potion_heals_before_moves_and_is_offered_only_where_it_helps() {
    let mut hurt = mon("Ally", &["splash"]);
    hurt.hp = 100;
    let mut fast_foe = mon("Foe", &["tackle"]);
    fast_foe.speed = 300;
    let mut b = battle(vec![hurt, mon("Bench", &["splash"])], vec![fast_foe]);
    b.set_bag(Side::Allies, vec!["potion".into(), "antidote".into()])
        .unwrap();
    let (p, _) = prompt(&mut b);
    assert_eq!(
        items(&p),
        [("potion".into(), id(Side::Allies, 0), None)],
        "full HP and no status rule out the rest"
    );
    let events = turn_with_item(&mut b, "potion", id(Side::Allies, 0));
    let used = events
        .iter()
        .position(|e| {
            *e == BattleEvent::ItemUsed {
                actor: id(Side::Allies, 0),
                item: "potion".into(),
                target: id(Side::Allies, 0),
                consumed: true,
            }
        })
        .unwrap();
    let foe_move = events
        .iter()
        .position(|e| matches!(e, BattleEvent::MoveUsed { .. }))
        .unwrap();
    assert!(used < foe_move, "items act before moves, even slower ones");
    assert!(said(&events, "Ally's HP was restored."), "{events:?}");
}

#[test]
fn revives_target_fainted_party_members() {
    let mut fainted = mon("Fainted", &["splash"]);
    fainted.hp = 0;
    let mut b = battle(
        vec![mon("Ally", &["splash"]), fainted],
        vec![mon("Foe", &["splash"])],
    );
    b.set_bag(Side::Allies, vec!["revive".into()]).unwrap();
    let (p, _) = prompt(&mut b);
    assert_eq!(items(&p), [("revive".into(), id(Side::Allies, 1), None)]);
    let events = turn_with_item(&mut b, "revive", id(Side::Allies, 1));
    assert_eq!(b.participants(Side::Allies)[1].hp, 250);
    assert!(said(&events, "Fainted was revived!"), "{events:?}");
}

#[test]
fn status_items_cure_and_full_heal_cures_confusion_too() {
    let mut poisoned = mon("Ally", &["splash"]);
    poisoned.status = Some(Status::Poisoned);
    poisoned.confused_turns = Some(3);
    let mut b = battle(vec![poisoned], vec![mon("Foe", &["splash"])]);
    b.set_bag(Side::Allies, vec!["full-heal".into(), "burn-heal".into()])
        .unwrap();
    let (p, _) = prompt(&mut b);
    assert_eq!(items(&p).len(), 1);
    answer(&mut b, &p, use_item("full-heal", id(Side::Allies, 0)));
    let (p, _) = prompt(&mut b);
    answer(&mut b, &p, uses("splash"));
    b.advance().unwrap();
    let ally = &b.participants(Side::Allies)[0];
    assert_eq!(ally.status, None);
    assert_eq!(ally.confused_turns, None);
}

#[test]
fn ethers_pick_a_move_and_elixirs_restore_all() {
    let mut tired = mon("Ally", &["tackle", "growl"]);
    tired.move_pp.insert("tackle".into(), 1);
    tired.move_pp.insert("growl".into(), 2);
    let mut b = battle(vec![tired], vec![mon("Foe", &["splash"])]);
    b.set_bag(Side::Allies, vec!["ether".into()]).unwrap();
    let (p, _) = prompt(&mut b);
    assert_eq!(items(&p).len(), 2, "one choice per move");
    answer(
        &mut b,
        &p,
        |c| matches!(c, Choice::UseItem { move_id: Some(m), .. } if m == "growl"),
    );
    let (p, _) = prompt(&mut b);
    answer(&mut b, &p, uses("splash"));
    b.advance().unwrap();
    let pp = &b.participants(Side::Allies)[0].move_pp;
    assert_eq!((pp["tackle"], pp["growl"]), (1, 12));

    b.set_bag(Side::Allies, vec!["max-elixir".into()]).unwrap();
    turn_with_item(&mut b, "max-elixir", id(Side::Allies, 0));
    let pp = &b.participants(Side::Allies)[0].move_pp;
    assert_eq!((pp["tackle"], pp["growl"]), (35, 40));
}

#[test]
fn x_items_raise_stages_and_bag_berries_reuse_held_berry_effects() {
    let mut hurt = mon("Ally", &["splash"]);
    hurt.hp = 400;
    let mut b = battle(vec![hurt], vec![mon("Foe", &["splash"])]);
    b.set_bag(Side::Allies, vec!["x-attack".into(), "oran-berry".into()])
        .unwrap();
    turn_with_item(&mut b, "x-attack", id(Side::Allies, 0));
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), 2);
    turn_with_item(&mut b, "oran-berry", id(Side::Allies, 0));
    assert_eq!(b.participants(Side::Allies)[0].hp, 410);
}

#[test]
fn escape_items_end_wild_battles_even_when_trapped() {
    let mut trapped = mon("Ally", &["splash"]);
    trapped.bound = Some(wilds_battle_engine::Bound {
        by: Some(id(Side::Foes, 0)),
        turns_left: 4,
        source: "Wrap".into(),
    });
    let mut b = battle(vec![trapped], vec![mon("Foe", &["splash"])]);
    b.set_wild_battle(true).unwrap();
    b.set_bag(Side::Allies, vec!["poke-doll".into()]).unwrap();
    let (p, _) = prompt(&mut b);
    assert!(!p.choices.iter().any(|c| matches!(c, Choice::Escape { .. })));
    answer(&mut b, &p, use_item("poke-doll", id(Side::Allies, 0)));
    let (p, _) = prompt(&mut b);
    answer(&mut b, &p, uses("splash"));
    let result = b.advance().unwrap();
    assert!(matches!(
        result.status,
        AdvanceStatus::Escaped { side: Side::Allies }
    ));
    assert!(said(&result.events, "Got away safely!"));

    for (wild, no_escape) in [(false, false), (true, true)] {
        let mut b = battle(
            vec![mon("Ally", &["splash"])],
            vec![mon("Foe", &["splash"])],
        );
        b.set_wild_battle(wild).unwrap();
        b.set_no_escape(no_escape);
        b.set_bag(Side::Allies, vec!["poke-doll".into()]).unwrap();
        assert!(items(&prompt(&mut b).0).is_empty());
    }
}

#[test]
fn host_items_can_reveal_disguises_and_keep_themselves() {
    let source = |id: &str| -> Option<MoveFile> {
        let text = match id {
            "lens" => {
                r#"return {id = "lens", name = "Lens", targets = "foe",
                usable = function(target) return target.uncatchable end,
                script = function(ctx) if ctx:reveal(ctx.target) then ctx:message("The disguise fell away!") end end}"#
            }
            "token" => {
                r#"return {id = "token", name = "Token", targets = "user",
                script = function(ctx) ctx:keep_item() end}"#
            }
            _ => return None,
        };
        Some(MoveFile {
            origin: format!("mod/{id}.lua"),
            text: text.into(),
        })
    };
    let catalog = MoveCatalog::with_sources(|_| None, source).unwrap();
    let mut disguised = mon("Foe", &["splash"]);
    disguised.uncatchable = true;
    disguised.disguise = Some(PokemonAppearance {
        name: "Decoy".into(),
        species_id: 1,
        species: "decoy".into(),
        form: None,
        gender: Default::default(),
    });
    let mut b = battle_with(catalog, vec![mon("Ally", &["splash"])], vec![disguised]);
    b.set_bag(Side::Allies, vec!["lens".into(), "token".into()])
        .unwrap();
    let events = turn_with_item(&mut b, "lens", id(Side::Foes, 0));
    assert!(said(&events, "The disguise fell away!"), "{events:?}");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, BattleEvent::AppearanceChanged { illusion: None, .. }))
    );
    let foe = &b.participants(Side::Foes)[0];
    assert!(!foe.uncatchable && foe.disguise.is_none());

    let events = turn_with_item(&mut b, "token", id(Side::Allies, 0));
    assert!(events.iter().any(|e| matches!(e,
        BattleEvent::ItemUsed { item, consumed: false, .. } if item == "token")));
}

#[test]
fn bags_reject_unknown_items_and_serve_either_side() {
    let mut b = battle(
        vec![mon("Ally", &["splash"])],
        vec![mon("Foe", &["splash"])],
    );
    assert!(
        b.set_bag(Side::Allies, vec!["no-such-item".into()])
            .is_err()
    );
    b.set_bag(Side::Foes, vec!["x-speed".into()]).unwrap();
    let (p, _) = prompt(&mut b);
    answer(&mut b, &p, uses("splash"));
    let (p, _) = prompt(&mut b);
    assert_eq!(p.actor, id(Side::Foes, 0));
    assert_eq!(items(&p), [("x-speed".into(), id(Side::Foes, 0), None)]);
}
