use wilds_battle_engine::{
    ActionSelection, AdvanceResult, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice,
    Condition, MoveCatalog, ParticipantId, Pokemon, PromptReason, Rule, Side, Stat, Status,
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
fn game(allies: Vec<Pokemon>, foes: Vec<Pokemon>, active: [Vec<usize>; 2]) -> Battle {
    let mut b = Battle::with_rng(Fixed, [allies, foes], MoveCatalog::builtin().unwrap()).unwrap();
    b.set_active(active).unwrap();
    b
}
fn choose(b: &mut Battle, actor: ParticipantId, f: impl Fn(&Choice) -> bool) -> AdvanceResult {
    let result = b.advance().unwrap();
    let AdvanceStatus::Awaiting(p) = &result.status else {
        panic!("no prompt");
    };
    assert_eq!(p.actor, actor);
    let choice = p.choices.iter().find(|c| f(c)).expect("choice");
    b.set_response(ActionSelection {
        prompt_id: p.id,
        choice_id: choice.id(),
    })
    .unwrap();
    result
}
fn pick(b: &mut Battle, actor: ParticipantId, name: &str) -> AdvanceResult {
    choose(
        b,
        actor,
        |c| matches!(c,Choice::UseMove{move_id,..}if move_id==name),
    )
}
fn switch(b: &mut Battle, actor: ParticipantId, to: usize) -> AdvanceResult {
    choose(
        b,
        actor,
        |c| matches!(c,Choice::Switch{to: got,..}if got.index==to),
    )
}

#[test]
fn switches_redirect_queued_attacks_to_the_field_slot_and_residuals_to_incoming() {
    let mut incoming = mon("Reserve", &["splash"]);
    incoming.status = Some(Status::Poisoned);
    let mut b = game(
        vec![mon("Lead", &["splash"]), incoming],
        vec![mon("Foe", &["tackle"])],
        [vec![0], vec![0]],
    );
    switch(&mut b, id(Side::Allies, 0), 1);
    pick(&mut b, id(Side::Foes, 0), "tackle");
    let result = b.advance().unwrap();
    assert_eq!(b.active_indices(Side::Allies), &[1]);
    assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
    assert!(b.participants(Side::Allies)[1].hp < 875);
    assert!(
        result
            .events
            .iter()
            .any(|e| matches!(e,BattleEvent::Switched{position:0,to,..}if *to==id(Side::Allies,1)))
    );
    assert!(
        !result
            .events
            .iter()
            .any(|e| matches!(e,BattleEvent::Damage{target,..}if *target==id(Side::Allies,0)))
    );
    let AdvanceStatus::Awaiting(p) = result.status else {
        panic!()
    };
    assert_eq!(p.actor, id(Side::Allies, 1));
    assert_eq!(b.position(id(Side::Allies, 0)), None);
    assert_eq!(b.position(id(Side::Allies, 1)), Some(0));
}

#[test]
fn switching_clears_volatile_state_but_preserves_party_state() {
    let mut lead = mon("Lead", &["splash"]);
    lead.status = Some(Status::Asleep);
    lead.status_turns = 3;
    lead.stages.insert(Stat::Defense, 2);
    lead.confused_turns = Some(3);
    lead.ability_suppressed = true;
    lead.conditions.push(Condition {
        name: "mark".into(),
        value: 1,
        turns_left: None,
        rules: vec![],
        group: None,
        end_message: None,
    });
    lead.item = Some("power-herb".into());
    lead.ate_berry = true;
    lead.consumed_items.insert("sitrus-berry".into());
    lead.move_pp.insert("splash".into(), 2);
    let mut b = game(
        vec![lead, mon("Reserve", &["splash"])],
        vec![mon("Foe", &["splash"])],
        [vec![0], vec![0]],
    );
    switch(&mut b, id(Side::Allies, 0), 1);
    pick(&mut b, id(Side::Foes, 0), "splash");
    b.advance().unwrap();
    let p = &b.participants(Side::Allies)[0];
    assert!(p.conditions.is_empty() && p.stages.is_empty() && p.confused_turns.is_none());
    assert!(!p.ability_suppressed);
    assert_eq!(p.status, Some(Status::Asleep));
    assert_eq!(p.status_turns, 3);
    assert_eq!(p.move_pp["splash"], 2);
    assert!(p.ate_berry);
    assert_eq!(p.item.as_deref(), Some("power-herb"));
}

#[test]
fn fainted_active_prompts_for_replacement_and_bench_keeps_the_battle_alive() {
    let mut lead = mon("Lead", &["splash"]);
    lead.hp = 1;
    let mut b = game(
        vec![lead, mon("Reserve", &["splash"])],
        vec![mon("Foe", &["tackle"])],
        [vec![0], vec![0]],
    );
    pick(&mut b, id(Side::Allies, 0), "splash");
    pick(&mut b, id(Side::Foes, 0), "tackle");
    let result = switch(&mut b, id(Side::Allies, 0), 1);
    let AdvanceStatus::Awaiting(p) = result.status else {
        panic!()
    };
    assert_eq!(p.reason, PromptReason::Replacement);
    assert!(p.choices.iter().all(|c| matches!(c, Choice::Switch { .. })));
    assert_eq!(b.turn(), 1);
    let next = b.advance().unwrap();
    assert_eq!(b.turn(), 1);
    assert!(
        next.events
            .iter()
            .any(|e| matches!(e, BattleEvent::Switched { .. }))
    );
    let AdvanceStatus::Awaiting(p) = next.status else {
        panic!()
    };
    assert_eq!(p.actor, id(Side::Allies, 1));
    assert_eq!(p.reason, PromptReason::Action);
}

#[test]
fn active_configuration_validates_and_double_choices_reserve_bench_members() {
    let mut b = game(
        vec![
            mon("A", &["splash"]),
            mon("B", &["splash"]),
            mon("C", &["splash"]),
        ],
        vec![mon("Foe", &["splash"])],
        [vec![0, 1], vec![0]],
    );
    assert!(b.set_active([vec![0, 0], vec![0]]).is_err());
    assert!(b.set_active([vec![9], vec![0]]).is_err());
    switch(&mut b, id(Side::Allies, 0), 2);
    let AdvanceStatus::Awaiting(p) = b.advance().unwrap().status else {
        panic!()
    };
    assert_eq!(p.actor, id(Side::Allies, 1));
    assert!(!p.choices.iter().any(|c| matches!(c, Choice::Switch { .. })));
    assert!(b.set_active([vec![0], vec![0]]).is_err());
}

#[test]
fn bench_is_not_a_target_and_sparkly_swirl_cures_the_party() {
    let mut lead = mon("Lead", &["sparkly-swirl"]);
    lead.status = Some(Status::Burned);
    let mut reserve = mon("Reserve", &["splash"]);
    reserve.status = Some(Status::Poisoned);
    reserve.hp = 100;
    reserve.item = Some("sitrus-berry".into());
    let mut enemy_bench = mon("Enemy reserve", &["splash"]);
    enemy_bench.ability = Some("unnerve".into());
    let mut b = game(
        vec![lead, reserve],
        vec![mon("Foe", &["splash"]), enemy_bench],
        [vec![0], vec![0]],
    );
    let prompt = pick(&mut b, id(Side::Allies, 0), "sparkly-swirl");
    let AdvanceStatus::Awaiting(p) = prompt.status else {
        panic!()
    };
    assert!(
        p.choices
            .iter()
            .all(|c| !matches!(c,Choice::UseMove{target,..}if target.index==1))
    );
    pick(&mut b, id(Side::Foes, 0), "splash");
    b.advance().unwrap();
    assert!(
        b.participants(Side::Allies)
            .iter()
            .all(|p| p.status.is_none())
    );
    assert_eq!(b.participants(Side::Allies)[1].hp, 100);
    assert_eq!(
        b.participants(Side::Allies)[1].item.as_deref(),
        Some("sitrus-berry")
    );
    assert_eq!(b.participants(Side::Foes)[1].move_pp["splash"], 20);
}

#[test]
fn bench_does_not_change_screen_scaling_or_adjacency() {
    let run = |bench| {
        let mut lead = mon("Lead", &["splash"]);
        lead.conditions.push(Condition {
            name: "screen".into(),
            value: 1,
            turns_left: None,
            rules: vec![Rule::DamageTaken {
                category: None,
                move_type: None,
                factor: 0.5,
                not_on_crit: true,
                multi_battle_factor: Some(2.0 / 3.0),
            }],
            group: None,
            end_message: None,
        });
        let mut allies = vec![lead];
        if bench {
            allies.push(mon("Reserve", &["splash"]));
        }
        let mut b = game(allies, vec![mon("Foe", &["tackle"])], [vec![0], vec![0]]);
        pick(&mut b, id(Side::Allies, 0), "splash");
        pick(&mut b, id(Side::Foes, 0), "tackle");
        b.advance().unwrap();
        b.participants(Side::Allies)[0].hp
    };
    assert_eq!(run(false), run(true));
    let b = game(
        vec![
            mon("A", &["splash"]),
            mon("Reserve", &["splash"]),
            mon("B", &["splash"]),
        ],
        vec![mon("Foe", &["splash"])],
        [vec![0, 2], vec![0]],
    );
    assert!(b.adjacent(id(Side::Allies, 0), id(Side::Allies, 2)));
    assert!(!b.adjacent(id(Side::Allies, 0), id(Side::Allies, 1)));
}

#[test]
fn linked_traps_expire_on_source_withdrawal_and_do_not_return_on_reentry() {
    for attack in ["anchor-shot", "spirit-shackle", "thousand-waves"] {
        let mut foe = mon("Foe", &["splash"]);
        foe.types = vec![wilds_battle_engine::PokemonType::Grass];
        let mut b = game(
            vec![
                mon("Lead", &[attack, "splash"]),
                mon("Reserve", &["splash"]),
            ],
            vec![foe, mon("Enemy reserve", &["splash"])],
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), attack);
        pick(&mut b, id(Side::Foes, 0), "splash");
        b.advance().unwrap();
        assert!(b.trapped(id(Side::Foes, 0)), "{attack}");
        switch(&mut b, id(Side::Allies, 0), 1);
        let result = pick(&mut b, id(Side::Foes, 0), "splash");
        let AdvanceStatus::Awaiting(p) = result.status else {
            panic!()
        };
        assert!(!p.choices.iter().any(|c| matches!(c, Choice::Switch { .. })));
        b.advance().unwrap();
        assert!(!b.trapped(id(Side::Foes, 0)), "{attack}");
        switch(&mut b, id(Side::Allies, 1), 0);
        pick(&mut b, id(Side::Foes, 0), "splash");
        b.advance().unwrap();
        assert!(!b.trapped(id(Side::Foes, 0)));
    }
}

#[test]
fn jaw_lock_links_both_participants_and_shed_shell_or_ghost_can_escape() {
    for ghost in [false, true] {
        let mut target = mon("Foe", &["splash"]);
        if ghost {
            target.types = vec![wilds_battle_engine::PokemonType::Ghost];
        } else {
            target.item = Some("shed-shell".into());
        }
        let mut b = game(
            vec![
                mon("Lead", &["jaw-lock", "splash"]),
                mon("Reserve", &["splash"]),
            ],
            vec![target, mon("Enemy reserve", &["splash"])],
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), "jaw-lock");
        pick(&mut b, id(Side::Foes, 0), "splash");
        b.advance().unwrap();
        assert!(b.trapped(id(Side::Allies, 0)));
        assert!(!b.trapped(id(Side::Foes, 0)));
        pick(&mut b, id(Side::Allies, 0), "splash");
        switch(&mut b, id(Side::Foes, 0), 1);
        b.advance().unwrap();
        assert!(!b.trapped(id(Side::Allies, 0)));
    }
}

#[test]
fn traps_and_binding_release_when_the_source_faints() {
    for attack in ["anchor-shot", "bind"] {
        let mut b = game(
            vec![
                mon("Lead", &[attack, "explosion"]),
                mon("Reserve", &["splash"]),
            ],
            vec![mon("Foe", &["splash"]), mon("Enemy reserve", &["splash"])],
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), attack);
        pick(&mut b, id(Side::Foes, 0), "splash");
        b.advance().unwrap();
        assert!(b.trapped(id(Side::Foes, 0)));
        pick(&mut b, id(Side::Allies, 0), "explosion");
        pick(&mut b, id(Side::Foes, 0), "splash");
        b.advance().unwrap();
        assert_eq!(b.participants(Side::Allies)[0].hp, 0);
        assert!(b.participants(Side::Foes)[0].hp > 0);
        assert!(!b.trapped(id(Side::Foes, 0)));
        assert!(b.participants(Side::Foes)[0].bound.is_none());
    }
}

#[test]
fn no_retreat_locks_once_except_when_already_linked_to_another_trapper() {
    let mut foe = mon("Foe", &["anchor-shot", "splash"]);
    foe.speed = 200;
    let mut b = game(
        vec![mon("Lead", &["no-retreat"]), mon("Reserve", &["splash"])],
        vec![foe, mon("Enemy reserve", &["splash"])],
        [vec![0], vec![0]],
    );
    for attack in ["anchor-shot", "splash"] {
        pick(&mut b, id(Side::Allies, 0), "no-retreat");
        pick(&mut b, id(Side::Foes, 0), attack);
        b.advance().unwrap();
    }
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), 2);
    assert!(
        b.participants(Side::Allies)[0]
            .condition("no_retreat")
            .is_none()
    );
    let mut b = game(
        vec![mon("Lead", &["no-retreat"]), mon("Reserve", &["splash"])],
        vec![mon("Foe", &["splash"])],
        [vec![0], vec![0]],
    );
    pick(&mut b, id(Side::Allies, 0), "no-retreat");
    pick(&mut b, id(Side::Foes, 0), "splash");
    let result = b.advance().unwrap();
    let AdvanceStatus::Awaiting(p) = result.status else {
        panic!()
    };
    assert!(!p.choices.iter().any(|c| matches!(c, Choice::Switch { .. })));
}

#[test]
fn drain_recovery_follows_the_source_field_slot_after_switching() {
    let mut target = mon("Foe", &["splash"]);
    target.conditions.push(Condition {
        name: "seed".into(),
        value: 1,
        turns_left: None,
        rules: vec![Rule::DrainEachTurn {
            fraction: 0.125,
            to: id(Side::Allies, 0),
            message: None,
        }],
        group: None,
        end_message: None,
    });
    let mut lead = mon("Lead", &["splash"]);
    lead.hp = 500;
    let mut reserve = mon("Reserve", &["splash"]);
    reserve.hp = 500;
    let mut b = game(vec![lead, reserve], vec![target], [vec![0], vec![0]]);
    switch(&mut b, id(Side::Allies, 0), 1);
    pick(&mut b, id(Side::Foes, 0), "splash");
    b.advance().unwrap();
    assert_eq!(b.participants(Side::Allies)[0].hp, 500);
    assert_eq!(b.participants(Side::Allies)[1].hp, 625);
}
