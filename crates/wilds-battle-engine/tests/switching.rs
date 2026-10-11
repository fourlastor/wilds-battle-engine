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

#[test]
fn phazing_cancels_the_outgoing_action_and_gives_residuals_to_the_replacement() {
    for attack in ["circle-throw", "dragon-tail"] {
        let mut lead = mon("Lead", &[attack]);
        lead.speed = 200;
        let mut reserve = mon("Enemy reserve", &["splash"]);
        reserve.status = Some(Status::Poisoned);
        let mut b = game(
            vec![lead],
            vec![mon("Foe", &["dragon-tail"]), reserve],
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), attack);
        pick(&mut b, id(Side::Foes, 0), "dragon-tail");
        let result = b.advance().unwrap();
        assert_eq!(b.active_indices(Side::Foes), &[1]);
        assert!(b.participants(Side::Foes)[0].hp < 1000);
        assert_eq!(b.participants(Side::Foes)[1].hp, 875);
        assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
        assert_eq!(b.participants(Side::Foes)[0].move_pp["dragon-tail"], 10);
        assert!(
            result
                .events
                .iter()
                .any(|e| matches!(e,BattleEvent::Switched{to,..}if *to==id(Side::Foes,1)))
        );
    }
}

#[test]
fn phazing_respects_suction_cups_and_anchoring_but_ignores_ordinary_traps() {
    for (ability, anchored, mold, switched) in [
        (None, false, false, true),
        (Some("suction-cups"), false, false, false),
        (Some("suction-cups"), false, true, true),
        (None, true, false, false),
    ] {
        let mut lead = mon("Lead", &["dragon-tail"]);
        if mold {
            lead.ability = Some("mold-breaker".into());
        }
        let mut foe = mon("Foe", &["splash"]);
        foe.ability = ability.map(str::to_owned);
        foe.conditions.push(Condition {
            name: "anchor".into(),
            value: 1,
            turns_left: None,
            rules: vec![if anchored {
                Rule::BlockForcedSwitch
            } else {
                Rule::Trapped
            }],
            group: None,
            end_message: None,
        });
        let mut b = game(
            vec![lead],
            vec![foe, mon("Enemy reserve", &["splash"])],
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), "dragon-tail");
        pick(&mut b, id(Side::Foes, 0), "splash");
        b.advance().unwrap();
        assert_eq!(
            b.active_indices(Side::Foes),
            &[usize::from(switched)],
            "{ability:?} {anchored} {mold}"
        );
        assert!(b.participants(Side::Foes)[0].hp < 1000);
    }
}

#[test]
fn phazing_needs_a_successful_hit_and_a_living_target_with_a_bench() {
    for (protected, immune, bench, hp, changed) in [
        (true, false, true, 1000, false),
        (false, true, true, 1000, false),
        (false, false, false, 1000, false),
        (false, false, true, 1, false),
        (false, false, true, 1000, true),
    ] {
        let mut foe = mon("Foe", &["protect", "splash"]);
        foe.hp = hp;
        if immune {
            foe.types = vec![wilds_battle_engine::PokemonType::Fairy];
        }
        let mut foes = vec![foe];
        if bench {
            foes.push(mon("Reserve", &["splash"]));
        }
        let mut b = game(
            vec![mon("Lead", &["dragon-tail"])],
            foes,
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), "dragon-tail");
        pick(
            &mut b,
            id(Side::Foes, 0),
            if protected { "protect" } else { "splash" },
        );
        let result = b.advance().unwrap();
        assert_eq!(
            result
                .events
                .iter()
                .any(|e| matches!(e, BattleEvent::Switched { .. })),
            changed
        );
    }
}

#[test]
fn pivots_pause_for_a_host_choice_and_resume_the_same_turn() {
    for attack in ["u-turn", "volt-switch", "flip-turn", "parting-shot"] {
        let mut lead = mon("Lead", &[attack]);
        lead.speed = 200;
        lead.conditions.push(Condition {
            name: "trapped".into(),
            value: 1,
            turns_left: None,
            rules: vec![Rule::Trapped],
            group: None,
            end_message: None,
        });
        let mut reserve = mon("Reserve", &["splash"]);
        reserve.status = Some(Status::Poisoned);
        let mut b = game(
            vec![lead, reserve],
            vec![mon("Foe", &["tackle"])],
            [vec![0], vec![0]],
        );
        let initial_pp = b.participants(Side::Allies)[0].move_pp[attack];
        pick(&mut b, id(Side::Allies, 0), attack);
        pick(&mut b, id(Side::Foes, 0), "tackle");
        let first = switch(&mut b, id(Side::Allies, 0), 1);
        let AdvanceStatus::Awaiting(p) = first.status else {
            panic!()
        };
        assert_eq!(p.reason, PromptReason::Pivot);
        assert_eq!(b.turn(), 1);
        assert_eq!(b.participants(Side::Allies)[1].hp, 1000);
        assert_eq!(
            b.participants(Side::Allies)[0].move_pp[attack],
            initial_pp - 1
        );
        let second = b.advance().unwrap();
        assert_eq!(b.turn(), 1);
        assert!(b.participants(Side::Allies)[1].hp < 875);
        assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
        assert_eq!(b.participants(Side::Foes)[0].move_pp["tackle"], 34);
        assert!(
            second
                .events
                .iter()
                .any(|e| matches!(e,BattleEvent::Damage{target,..}if *target==id(Side::Allies,1)))
        );
    }
}

#[test]
fn pivot_does_not_trigger_when_its_hit_is_blocked_or_immune() {
    for (attack, kind) in [
        ("u-turn", None),
        (
            "volt-switch",
            Some(wilds_battle_engine::PokemonType::Ground),
        ),
        ("flip-turn", None),
    ] {
        let mut foe = mon("Foe", &["protect", "splash"]);
        if let Some(kind) = kind {
            foe.types = vec![kind];
        }
        let mut b = game(
            vec![mon("Lead", &[attack]), mon("Reserve", &["splash"])],
            vec![foe],
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), attack);
        pick(
            &mut b,
            id(Side::Foes, 0),
            if kind.is_some() { "splash" } else { "protect" },
        );
        let result = b.advance().unwrap();
        let AdvanceStatus::Awaiting(p) = result.status else {
            panic!()
        };
        assert_eq!(p.reason, PromptReason::Action);
        assert_eq!(b.active_indices(Side::Allies), &[0]);
    }
}

#[test]
fn parting_shot_needs_a_stat_change_and_respects_soundproof() {
    for (capped, soundproof, blocked, mold, swaps) in [
        (true, false, false, false, false),
        (false, true, false, false, false),
        (false, false, true, false, false),
        (false, true, false, true, true),
    ] {
        let mut lead = mon("Lead", &["parting-shot"]);
        lead.speed = 200;
        if mold {
            lead.ability = Some("mold-breaker".into());
        }
        let mut foe = mon("Foe", &["splash"]);
        if capped {
            foe.stages.insert(Stat::Attack, -6);
            foe.stages.insert(Stat::SpAttack, -6);
        }
        if soundproof {
            foe.ability = Some("soundproof".into());
        }
        if blocked {
            foe.conditions.push(Condition {
                name: "mist".into(),
                value: 1,
                turns_left: None,
                rules: vec![Rule::BlockStatDrops],
                group: None,
                end_message: None,
            });
        }
        let mut b = game(
            vec![lead, mon("Reserve", &["splash"])],
            vec![foe],
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), "parting-shot");
        pick(&mut b, id(Side::Foes, 0), "splash");
        let result = b.advance().unwrap();
        let AdvanceStatus::Awaiting(p) = result.status else {
            panic!()
        };
        assert_eq!(p.reason == PromptReason::Pivot, swaps);
    }
}

#[test]
fn two_pivots_in_one_turn_resume_each_other_without_replaying_actions() {
    let mut lead = mon("Lead", &["u-turn"]);
    lead.speed = 200;
    let mut b = game(
        vec![lead, mon("Reserve", &["splash"])],
        vec![
            mon("Foe", &["volt-switch"]),
            mon("Enemy reserve", &["splash"]),
        ],
        [vec![0], vec![0]],
    );
    pick(&mut b, id(Side::Allies, 0), "u-turn");
    pick(&mut b, id(Side::Foes, 0), "volt-switch");
    switch(&mut b, id(Side::Allies, 0), 1);
    let second = switch(&mut b, id(Side::Foes, 0), 1);
    let AdvanceStatus::Awaiting(p) = second.status else {
        panic!()
    };
    assert_eq!(p.reason, PromptReason::Pivot);
    b.advance().unwrap();
    assert_eq!(b.turn(), 1);
    assert_eq!(b.active_indices(Side::Allies), &[1]);
    assert_eq!(b.active_indices(Side::Foes), &[1]);
    assert_eq!(b.participants(Side::Allies)[0].move_pp["u-turn"], 19);
    assert_eq!(b.participants(Side::Foes)[0].move_pp["volt-switch"], 19);
    assert!(b.participants(Side::Allies)[1].hp < 1000);
    assert_eq!(b.participants(Side::Foes)[1].hp, 1000);
}

#[test]
fn pursuit_intercepts_voluntary_and_pivot_switches_once_at_double_power() {
    let run =
        |pivot: bool, leaving: bool| {
            let mut lead = mon("Lead", &["u-turn", "splash"]);
            lead.speed = 200;
            let mut b = game(
                vec![lead, mon("Reserve", &["splash"])],
                vec![mon("Foe", &["pursuit"])],
                [vec![0], vec![0]],
            );
            if leaving && !pivot {
                switch(&mut b, id(Side::Allies, 0), 1);
            } else {
                pick(
                    &mut b,
                    id(Side::Allies, 0),
                    if pivot { "u-turn" } else { "splash" },
                );
            }
            pick(&mut b, id(Side::Foes, 0), "pursuit");
            if pivot {
                let first = switch(&mut b, id(Side::Allies, 0), 1);
                assert!(first.events.iter().any(
                    |e| matches!(e,BattleEvent::Damage{target,..}if *target==id(Side::Allies,0))
                ));
            }
            b.advance().unwrap();
            assert_eq!(b.participants(Side::Foes)[0].move_pp["pursuit"], 19);
            assert_eq!(b.participants(Side::Allies)[1].hp, 1000);
            1000 - b.participants(Side::Allies)[0].hp
        };
    let ordinary = run(false, false);
    let switched = run(false, true);
    let pivot = run(true, true);
    assert!(switched > ordinary * 17 / 10);
    assert_eq!(switched, pivot);
}

#[test]
fn pursuit_can_faint_a_switcher_before_replacement_and_cannot_miss_that_switch() {
    for hp in [1, 1000] {
        let mut lead = mon("Lead", &["splash"]);
        lead.hp = hp;
        lead.stages.insert(Stat::Evasion, 6);
        let mut foe = mon("Foe", &["pursuit"]);
        foe.stages.insert(Stat::Accuracy, -6);
        let mut b = game(
            vec![lead, mon("Reserve", &["splash"])],
            vec![foe],
            [vec![0], vec![0]],
        );
        switch(&mut b, id(Side::Allies, 0), 1);
        pick(&mut b, id(Side::Foes, 0), "pursuit");
        let result = b.advance().unwrap();
        assert!(b.participants(Side::Allies)[0].hp < hp);
        assert_eq!(b.participants(Side::Allies)[1].hp, 1000);
        assert_eq!(b.participants(Side::Foes)[0].move_pp["pursuit"], 19);
        let AdvanceStatus::Awaiting(p) = result.status else {
            panic!()
        };
        if hp == 1 {
            assert_eq!(p.reason, PromptReason::Replacement);
            assert!(
                !result
                    .events
                    .iter()
                    .any(|e| matches!(e, BattleEvent::Switched { .. }))
            );
        } else {
            assert_eq!(p.actor, id(Side::Allies, 1));
        }
    }
}

#[test]
fn pursuit_that_already_acted_does_not_intercept_a_later_pivot() {
    let mut foe = mon("Foe", &["pursuit"]);
    foe.speed = 200;
    let mut b = game(
        vec![mon("Lead", &["u-turn"]), mon("Reserve", &["splash"])],
        vec![foe],
        [vec![0], vec![0]],
    );
    pick(&mut b, id(Side::Allies, 0), "u-turn");
    pick(&mut b, id(Side::Foes, 0), "pursuit");
    let first = switch(&mut b, id(Side::Allies, 0), 1);
    let hp = b.participants(Side::Allies)[0].hp;
    let second = b.advance().unwrap();
    assert_eq!(b.participants(Side::Allies)[0].hp, hp);
    assert_eq!(b.participants(Side::Foes)[0].move_pp["pursuit"], 19);
    assert_eq!(
        first
            .events
            .iter()
            .chain(second.events.iter())
            .filter(|e| matches!(e,BattleEvent::Damage{target,..}if *target==id(Side::Allies,0)))
            .count(),
        1
    );
}

#[test]
fn an_intercepted_pursuit_still_checks_sleep_once() {
    let mut foe = mon("Foe", &["pursuit"]);
    foe.status = Some(Status::Asleep);
    foe.status_turns = 3;
    let mut b = game(
        vec![mon("Lead", &["splash"]), mon("Reserve", &["splash"])],
        vec![foe],
        [vec![0], vec![0]],
    );
    switch(&mut b, id(Side::Allies, 0), 1);
    pick(&mut b, id(Side::Foes, 0), "pursuit");
    b.advance().unwrap();
    assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
    assert_eq!(b.participants(Side::Allies)[1].hp, 1000);
    assert_eq!(b.participants(Side::Foes)[0].status_turns, 2);
    assert_eq!(b.participants(Side::Foes)[0].move_pp["pursuit"], 20);
}

#[test]
fn sky_drop_action_timing_depends_on_when_the_carried_pokemon_is_released() {
    for fast in [false, true] {
        let mut lead = mon("Carrier", &["sky-drop"]);
        lead.speed = if fast { 200 } else { 50 };
        lead.item = Some("power-herb".into());
        let mut b = game(
            vec![lead],
            vec![mon("Foe", &["tackle"])],
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), "sky-drop");
        pick(&mut b, id(Side::Foes, 0), "tackle");
        b.advance().unwrap();
        assert!(b.participants(Side::Allies)[0].carrying.is_some());
        assert!(b.participants(Side::Foes)[0].carried_by.is_some());
        let hp = b.participants(Side::Allies)[0].hp;
        assert_eq!(
            b.participants(Side::Foes)[0].move_pp["tackle"],
            if fast { 35 } else { 34 }
        );
        pick(&mut b, id(Side::Foes, 0), "tackle");
        b.advance().unwrap();
        assert_eq!(b.participants(Side::Foes)[0].move_pp["tackle"], 34);
        assert_eq!(b.participants(Side::Allies)[0].hp < hp, fast);
        assert!(b.participants(Side::Foes)[0].hp < 1000);
        assert!(b.participants(Side::Allies)[0].carrying.is_none());
        assert!(!b.participants(Side::Foes)[0].semi_invulnerable);
        assert_eq!(b.participants(Side::Allies)[0].move_pp["sky-drop"], 9);
        assert_eq!(
            b.participants(Side::Allies)[0].item.as_deref(),
            Some("power-herb")
        );
    }
}

#[test]
fn sky_drop_fails_against_heavy_or_protected_targets() {
    for (heavy, protect) in [(true, false), (false, true)] {
        let mut lead = mon("Carrier", &["sky-drop"]);
        lead.speed = 200;
        let mut foe = mon("Foe", &["protect", "splash"]);
        foe.weight = if heavy { 200.0 } else { 199.9 };
        let mut b = game(vec![lead], vec![foe], [vec![0], vec![0]]);
        pick(&mut b, id(Side::Allies, 0), "sky-drop");
        pick(
            &mut b,
            id(Side::Foes, 0),
            if protect { "protect" } else { "splash" },
        );
        b.advance().unwrap();
        assert!(b.participants(Side::Allies)[0].carrying.is_none());
        assert!(
            b.participants(Side::Allies)[0]
                .script_continuation
                .is_none()
        );
        assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    }
}

#[test]
fn sky_drop_carries_flying_targets_without_damaging_them_and_blocks_ghost_escape() {
    let mut lead = mon("Carrier", &["sky-drop"]);
    lead.speed = 200;
    let mut foe = mon("Foe", &["splash"]);
    foe.types = vec![
        wilds_battle_engine::PokemonType::Flying,
        wilds_battle_engine::PokemonType::Ghost,
    ];
    foe.item = Some("shed-shell".into());
    let mut b = game(
        vec![lead],
        vec![foe, mon("Reserve", &["splash"])],
        [vec![0], vec![0]],
    );
    pick(&mut b, id(Side::Allies, 0), "sky-drop");
    pick(&mut b, id(Side::Foes, 0), "splash");
    let result = b.advance().unwrap();
    assert!(b.trapped(id(Side::Foes, 0)));
    let AdvanceStatus::Awaiting(p) = result.status else {
        panic!()
    };
    assert!(!p.choices.iter().any(|c| matches!(c, Choice::Switch { .. })));
    pick(&mut b, id(Side::Foes, 0), "splash");
    b.advance().unwrap();
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    assert!(!b.trapped(id(Side::Foes, 0)));
}

#[test]
fn gravity_and_smack_down_release_the_pair_and_cancel_the_queued_drop() {
    for interrupt in ["gravity", "smack-down"] {
        let mut lead = mon("Carrier", &["sky-drop", "splash"]);
        lead.speed = 150;
        let mut partner = mon("Partner", &["splash", interrupt]);
        partner.speed = 200;
        let mut b = game(
            vec![lead],
            vec![mon("Foe", &["tackle"]), partner],
            [vec![0], vec![0, 1]],
        );
        pick(&mut b, id(Side::Allies, 0), "sky-drop");
        pick(&mut b, id(Side::Foes, 0), "tackle");
        pick(&mut b, id(Side::Foes, 1), "splash");
        b.advance().unwrap();
        assert!(b.participants(Side::Allies)[0].carrying.is_some());
        pick(&mut b, id(Side::Foes, 0), "tackle");
        pick(&mut b, id(Side::Foes, 1), interrupt);
        b.advance().unwrap();
        assert!(b.participants(Side::Allies)[0].carrying.is_none());
        assert!(
            b.participants(Side::Allies)[0]
                .script_continuation
                .is_none()
        );
        assert!(b.participants(Side::Foes)[0].carried_by.is_none());
        assert!(!b.participants(Side::Allies)[0].semi_invulnerable);
        assert!(!b.participants(Side::Foes)[0].semi_invulnerable);
        assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
        assert_eq!(b.participants(Side::Allies)[0].move_pp["sky-drop"], 9);
    }
}

#[test]
fn fainting_either_carried_participant_releases_both() {
    for carrier_faints in [false, true] {
        let mut lead = mon("Carrier", &["sky-drop"]);
        lead.speed = 200;
        let mut foe = mon("Foe", &["splash"]);
        let poisoned = if carrier_faints { &mut lead } else { &mut foe };
        poisoned.hp = 1;
        poisoned.status = Some(Status::Poisoned);
        let mut b = game(
            vec![lead, mon("Reserve", &["splash"])],
            vec![foe, mon("Enemy reserve", &["splash"])],
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), "sky-drop");
        pick(&mut b, id(Side::Foes, 0), "splash");
        b.advance().unwrap();
        assert!(b.participants(Side::Allies)[0].carrying.is_none());
        assert!(
            b.participants(Side::Allies)[0]
                .script_continuation
                .is_none()
        );
        assert!(b.participants(Side::Foes)[0].carried_by.is_none());
        assert!(!b.participants(Side::Allies)[0].semi_invulnerable);
        assert!(!b.participants(Side::Foes)[0].semi_invulnerable);
    }
}

#[test]
fn sky_drop_releases_its_target_when_paralysis_interrupts_the_carrier() {
    use std::{cell::Cell, rc::Rc};
    struct Controlled(Rc<Cell<f32>>);
    impl BattleRng for Controlled {
        fn next_u64(&mut self) -> u64 {
            0
        }
        fn fraction(&mut self) -> f32 {
            self.0.get()
        }
    }
    let dice = Rc::new(Cell::new(0.5));
    let mut lead = mon("Carrier", &["sky-drop"]);
    lead.speed = 800;
    lead.status = Some(Status::Paralyzed);
    let mut b = Battle::with_rng(
        Controlled(dice.clone()),
        [vec![lead], vec![mon("Foe", &["tackle"])]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    pick(&mut b, id(Side::Allies, 0), "sky-drop");
    pick(&mut b, id(Side::Foes, 0), "tackle");
    b.advance().unwrap();
    assert!(b.participants(Side::Allies)[0].carrying.is_some());
    dice.set(0.0);
    pick(&mut b, id(Side::Foes, 0), "tackle");
    b.advance().unwrap();
    assert!(b.participants(Side::Allies)[0].carrying.is_none());
    assert!(b.participants(Side::Foes)[0].carried_by.is_none());
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    assert_eq!(b.participants(Side::Allies)[0].move_pp["sky-drop"], 9);
}

#[test]
fn sky_drop_lift_ignores_accuracy_but_a_missed_drop_still_releases_both() {
    let mut lead = mon("Carrier", &["sky-drop"]);
    lead.speed = 200;
    lead.stages.insert(Stat::Accuracy, -6);
    let mut foe = mon("Foe", &["splash"]);
    foe.stages.insert(Stat::Evasion, 6);
    let mut b = game(vec![lead], vec![foe], [vec![0], vec![0]]);
    pick(&mut b, id(Side::Allies, 0), "sky-drop");
    pick(&mut b, id(Side::Foes, 0), "splash");
    b.advance().unwrap();
    assert!(b.participants(Side::Allies)[0].carrying.is_some());
    pick(&mut b, id(Side::Foes, 0), "splash");
    b.advance().unwrap();
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    assert!(b.participants(Side::Allies)[0].carrying.is_none());
    assert!(!b.participants(Side::Foes)[0].semi_invulnerable);
}

#[test]
fn an_air_reaching_hit_can_damage_the_carrier_but_cannot_force_it_out() {
    let mut lead = mon("Carrier", &["sky-drop"]);
    lead.conditions.push(Condition {
        name: "quick".into(),
        value: 1,
        turns_left: None,
        rules: vec![Rule::MovePriority { amount: 2 }],
        group: None,
        end_message: None,
    });
    let mut partner = mon("Partner", &["dragon-tail"]);
    partner.conditions.push(Condition {
        name: "aim".into(),
        value: 1,
        turns_left: None,
        rules: vec![
            Rule::AlwaysHit { against: None },
            Rule::MovePriority { amount: 7 },
        ],
        group: None,
        end_message: None,
    });
    let mut b = game(
        vec![lead, mon("Reserve", &["splash"])],
        vec![mon("Foe", &["splash"]), partner],
        [vec![0], vec![0, 1]],
    );
    pick(&mut b, id(Side::Allies, 0), "sky-drop");
    pick(&mut b, id(Side::Foes, 0), "splash");
    pick(&mut b, id(Side::Foes, 1), "dragon-tail");
    b.advance().unwrap();
    assert_eq!(b.active_indices(Side::Allies), &[0]);
    assert!(b.participants(Side::Allies)[0].hp < 1000);
    assert!(b.participants(Side::Allies)[0].carrying.is_some());
}

#[test]
fn entry_abilities_activate_for_initial_leads_and_each_reentry() {
    let mut lead = mon("Lead", &["splash"]);
    lead.ability = Some("intimidate".into());
    let mut b = game(
        vec![lead, mon("Bench", &["splash"])],
        vec![mon("Foe", &["splash"])],
        [vec![0], vec![0]],
    );
    let first = b.advance().unwrap();
    assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Attack), -1);
    assert!(
        first
            .events
            .iter()
            .any(|e| matches!(e, BattleEvent::StatChange { .. }))
    );
    b.advance().unwrap();
    assert_eq!(
        b.participants(Side::Foes)[0].stage(Stat::Attack),
        -1,
        "pending prompts do not reactivate abilities"
    );
    switch(&mut b, id(Side::Allies, 0), 1);
    pick(&mut b, id(Side::Foes, 0), "splash");
    b.advance().unwrap();
    switch(&mut b, id(Side::Allies, 1), 0);
    pick(&mut b, id(Side::Foes, 0), "splash");
    b.advance().unwrap();
    assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Attack), -2);
}

#[test]
fn intimidate_respects_adjacency_bench_suppression_and_immunity() {
    let mut lead = mon("Lead", &["splash"]);
    lead.ability = Some("intimidate".into());
    let mut immune = mon("Immune", &["splash"]);
    immune.ability = Some("clear-body".into());
    let mut b = game(
        vec![lead.clone()],
        vec![
            mon("Far", &["splash"]),
            mon("Near", &["splash"]),
            immune,
            mon("Bench", &["splash"]),
        ],
        [vec![0], vec![0, 1, 2]],
    );
    b.advance().unwrap();
    assert_eq!(
        b.participants(Side::Foes)
            .iter()
            .map(|p| p.stage(Stat::Attack))
            .collect::<Vec<_>>(),
        [0, -1, 0, 0]
    );
    lead.ability_suppressed = true;
    let mut b = game(
        vec![lead],
        vec![mon("Foe", &["splash"])],
        [vec![0], vec![0]],
    );
    b.advance().unwrap();
    assert_eq!(b.participants(Side::Foes)[0].stage(Stat::Attack), 0);
}

#[test]
fn withdrawal_abilities_run_before_suppression_resets_and_never_revive() {
    for ability in ["regenerator", "natural-cure"] {
        for (hp, suppressed) in [(300, false), (300, true), (0, false)] {
            let mut lead = mon("Lead", &["splash"]);
            lead.hp = hp;
            lead.status = Some(Status::Asleep);
            lead.status_turns = 5;
            lead.ability = Some(ability.into());
            lead.ability_suppressed = suppressed;
            let mut b = game(
                vec![lead, mon("Bench", &["splash"])],
                vec![mon("Foe", &["splash"])],
                [vec![0], vec![0]],
            );
            switch(&mut b, id(Side::Allies, 0), 1);
            if hp > 0 {
                pick(&mut b, id(Side::Foes, 0), "splash");
            }
            b.advance().unwrap();
            let outgoing = &b.participants(Side::Allies)[0];
            let active = hp > 0 && !suppressed;
            assert_eq!(
                outgoing.hp,
                if active && ability == "regenerator" {
                    633
                } else {
                    hp
                }
            );
            assert_eq!(
                outgoing.status,
                if active && ability == "natural-cure" {
                    None
                } else {
                    Some(Status::Asleep)
                }
            );
        }
    }
}

#[test]
fn forced_switches_activate_withdrawal_and_entry_abilities() {
    let mut outgoing = mon("Outgoing", &["splash"]);
    outgoing.ability = Some("regenerator".into());
    outgoing.hp = 500;
    let mut incoming = mon("Incoming", &["splash"]);
    incoming.ability = Some("intimidate".into());
    let mut b = game(
        vec![mon("User", &["dragon-tail"])],
        vec![outgoing, incoming],
        [vec![0], vec![0]],
    );
    pick(&mut b, id(Side::Allies, 0), "dragon-tail");
    pick(&mut b, id(Side::Foes, 0), "splash");
    b.advance().unwrap();
    assert_eq!(b.active_indices(Side::Foes), &[1]);
    assert!(b.participants(Side::Foes)[0].hp > 500);
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), -1);
}

#[test]
fn stockpile_and_swallow_repay_successful_changes_through_stage_abilities() {
    for (ability, stage) in [("contrary", -1), ("simple", 2)] {
        let mut user = mon("User", &["stockpile", "swallow"]);
        user.ability = Some(ability.into());
        let mut b = game(
            vec![user],
            vec![mon("Foe", &["splash"])],
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), "stockpile");
        pick(&mut b, id(Side::Foes, 0), "splash");
        b.advance().unwrap();
        assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Defense), stage);
        pick(&mut b, id(Side::Allies, 0), "swallow");
        pick(&mut b, id(Side::Foes, 0), "splash");
        b.advance().unwrap();
        assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Defense), 0);
        assert_eq!(b.participants(Side::Allies)[0].stage(Stat::SpDefense), 0);
    }
}

#[test]
fn spikes_layers_cap_and_damage_only_the_incoming_grounded_member() {
    for layers in 1..=4 {
        let mut b = game(
            vec![mon("Setter", &["spikes"])],
            vec![mon("Lead", &["splash"]), mon("Reserve", &["splash"])],
            [vec![0], vec![0]],
        );
        for _ in 0..layers {
            pick(&mut b, id(Side::Allies, 0), "spikes");
            pick(&mut b, id(Side::Foes, 0), "splash");
            b.advance().unwrap();
        }
        assert_eq!(b.side_conditions(Side::Foes)[0].value, layers.min(3));
        pick(&mut b, id(Side::Allies, 0), "spikes");
        switch(&mut b, id(Side::Foes, 0), 1);
        b.advance().unwrap();
        assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
        assert_eq!(
            b.participants(Side::Foes)[1].hp,
            1000 - [0, 125, 166, 250][layers.min(3) as usize]
        );
    }
}

#[test]
fn toxic_spikes_poison_absorption_airborne_and_steel_immunity() {
    use wilds_battle_engine::PokemonType;
    for (kind, layers, expected, removed) in [
        (PokemonType::Normal, 1, Some(Status::Poisoned), false),
        (PokemonType::Normal, 2, Some(Status::BadlyPoisoned), false),
        (PokemonType::Flying, 2, None, false),
        (PokemonType::Steel, 2, None, false),
        (PokemonType::Poison, 2, None, true),
    ] {
        let mut reserve = mon("Reserve", &["splash"]);
        reserve.types = vec![kind];
        let mut b = game(
            vec![mon("Setter", &["toxic-spikes", "splash"])],
            vec![mon("Lead", &["splash"]), reserve],
            [vec![0], vec![0]],
        );
        for _ in 0..layers {
            pick(&mut b, id(Side::Allies, 0), "toxic-spikes");
            pick(&mut b, id(Side::Foes, 0), "splash");
            b.advance().unwrap();
        }
        pick(&mut b, id(Side::Allies, 0), "splash");
        switch(&mut b, id(Side::Foes, 0), 1);
        b.advance().unwrap();
        assert_eq!(b.participants(Side::Foes)[1].status, expected);
        assert_eq!(b.side_conditions(Side::Foes).is_empty(), removed);
    }
}

#[test]
fn sticky_web_uses_stage_abilities_and_grounding_on_entry() {
    for (ability, gravity, expected) in [
        ("contrary", false, 1),
        ("simple", false, -2),
        ("clear-body", false, 0),
        ("levitate", false, 0),
        ("levitate", true, -1),
    ] {
        let mut reserve = mon("Reserve", &["splash"]);
        reserve.ability = Some(ability.into());
        let mut b = game(
            vec![mon("Setter", &["sticky-web", "gravity", "splash"])],
            vec![mon("Lead", &["splash"]), reserve],
            [vec![0], vec![0]],
        );
        pick(&mut b, id(Side::Allies, 0), "sticky-web");
        pick(&mut b, id(Side::Foes, 0), "splash");
        b.advance().unwrap();
        if gravity {
            pick(&mut b, id(Side::Allies, 0), "gravity");
            pick(&mut b, id(Side::Foes, 0), "splash");
            b.advance().unwrap();
        }
        pick(&mut b, id(Side::Allies, 0), "sticky-web");
        switch(&mut b, id(Side::Foes, 0), 1);
        b.advance().unwrap();
        assert_eq!(
            b.participants(Side::Foes)[1].stage(Stat::Speed),
            expected,
            "{ability} gravity={gravity}"
        );
    }
}

#[test]
fn stealth_rock_uses_type_effectiveness_and_knockout_prevents_entry_ability() {
    use wilds_battle_engine::PokemonType;
    let mut reserve = mon("Reserve", &["splash"]);
    reserve.types = vec![PokemonType::Fire, PokemonType::Flying];
    reserve.hp = 500;
    reserve.ability = Some("intimidate".into());
    let mut b = game(
        vec![mon("Setter", &["stealth-rock", "splash"])],
        vec![mon("Lead", &["splash"]), reserve],
        [vec![0], vec![0]],
    );
    pick(&mut b, id(Side::Allies, 0), "stealth-rock");
    pick(&mut b, id(Side::Foes, 0), "splash");
    b.advance().unwrap();
    pick(&mut b, id(Side::Allies, 0), "splash");
    switch(&mut b, id(Side::Foes, 0), 1);
    let result = b.advance().unwrap();
    assert_eq!(b.participants(Side::Foes)[1].hp, 0);
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), 0);
    let AdvanceStatus::Awaiting(prompt) = result.status else {
        panic!()
    };
    assert_eq!(prompt.reason, PromptReason::Replacement);
}

#[test]
fn rapid_spin_removes_real_entry_hazards() {
    let mut b = game(
        vec![mon(
            "Setter",
            &[
                "spikes",
                "toxic-spikes",
                "stealth-rock",
                "sticky-web",
                "splash",
            ],
        )],
        vec![mon("Spinner", &["rapid-spin", "splash"])],
        [vec![0], vec![0]],
    );
    for hazard in ["spikes", "toxic-spikes", "stealth-rock", "sticky-web"] {
        pick(&mut b, id(Side::Allies, 0), hazard);
        pick(&mut b, id(Side::Foes, 0), "splash");
        b.advance().unwrap();
    }
    assert_eq!(b.side_conditions(Side::Foes).len(), 4);
    pick(&mut b, id(Side::Allies, 0), "splash");
    pick(&mut b, id(Side::Foes, 0), "rapid-spin");
    b.advance().unwrap();
    assert!(b.side_conditions(Side::Foes).is_empty());
}
