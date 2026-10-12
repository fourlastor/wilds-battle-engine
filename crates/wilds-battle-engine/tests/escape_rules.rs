use wilds_battle_engine::{
    ActionSelection, AdvanceResult, AdvanceStatus, Battle, BattleEvent, BattleRng, Bound, Choice,
    Condition, ContinuationTarget, LockedMove, MoveCatalog, ParticipantId, Pokemon, PokemonType,
    Prompt, PromptReason, Rule, ScriptContinuation, Side, Stat, Status,
};

struct Dice(u8);
impl BattleRng for Dice {
    fn next_u64(&mut self) -> u64 {
        u64::from(self.0)
    }
    fn fraction(&mut self) -> f32 {
        0.5
    }
}
fn id(side: Side) -> ParticipantId {
    ParticipantId { side, index: 0 }
}
fn mon(name: &str, speed: u16) -> Pokemon {
    let mut p = Pokemon::new(name, vec!["tackle".into()]);
    p.max_hp = 1000;
    p.hp = 1000;
    p.speed = speed;
    p
}
fn game(allies: Vec<Pokemon>, foes: Vec<Pokemon>, die: u8, wild: bool) -> Battle {
    let mut b =
        Battle::with_rng(Dice(die), [allies, foes], MoveCatalog::builtin().unwrap()).unwrap();
    b.set_wild_battle(wild).unwrap();
    b
}
fn prompt(b: &mut Battle) -> Prompt {
    let AdvanceStatus::Awaiting(p) = b.advance().unwrap().status else {
        panic!("expected prompt")
    };
    p
}
fn choose(b: &mut Battle, p: &Prompt, predicate: impl Fn(&Choice) -> bool) {
    let choice = p.choices.iter().find(|c| predicate(c)).unwrap();
    b.set_response(ActionSelection {
        prompt_id: p.id,
        choice_id: choice.id(),
    })
    .unwrap();
}
fn flee(c: &Choice) -> bool {
    matches!(c, Choice::Escape { .. })
}
fn movement(c: &Choice) -> bool {
    matches!(c, Choice::UseMove { .. })
}
fn attempt(b: &mut Battle) -> AdvanceResult {
    let p = prompt(b);
    assert_eq!(p.actor.side, Side::Allies);
    choose(b, &p, flee);
    // All opposing actions still come from the host.
    loop {
        let result = b.advance().unwrap();
        if let AdvanceStatus::Awaiting(p) = &result.status
            && p.actor.side == Side::Foes
        {
            assert!(!p.choices.iter().any(flee));
            choose(b, p, movement);
            continue;
        }
        return result;
    }
}
fn escaped(result: &AdvanceResult) -> bool {
    result.status == AdvanceStatus::Escaped { side: Side::Allies }
}

#[test]
fn escape_is_opt_in_and_setup_cannot_change_after_start() {
    for ability in [None, Some("run-away")] {
        let mut user = mon("Runner", 1);
        user.ability = ability.map(str::to_owned);
        let mut b = game(vec![user], vec![mon("Foe", 100)], 255, false);
        assert!(!prompt(&mut b).choices.iter().any(flee));
        assert!(b.set_wild_battle(true).is_err());
    }
}

#[test]
fn run_away_guarantees_escape_without_attacks_pp_or_residuals() {
    let mut user = mon("Runner", 1);
    user.ability = Some("run-away".into());
    user.status = Some(Status::BadlyPoisoned);
    let mut target = mon("Foe", 500);
    target.moves = vec!["quick-attack".into()];
    let mut b = game(vec![user], vec![target], 255, true);
    let pp = b.participants(Side::Allies)[0].move_pp.clone();
    let foe_pp = b.participants(Side::Foes)[0].move_pp.clone();
    let result = attempt(&mut b);
    assert!(escaped(&result));
    assert!(result.events.contains(&BattleEvent::EscapeAttempt {
        actor: id(Side::Allies),
        success: true
    }));
    assert!(
        !result
            .events
            .iter()
            .any(|e| matches!(e, BattleEvent::Damage { .. } | BattleEvent::End { .. }))
    );
    assert_eq!(b.participants(Side::Allies)[0].hp, 1000);
    assert_eq!(b.participants(Side::Allies)[0].move_pp, pp);
    assert_eq!(b.participants(Side::Foes)[0].move_pp, foe_pp);
    assert_eq!(b.turn(), 1);
    let again = b.advance().unwrap();
    assert!(escaped(&again));
    assert!(again.events.is_empty());
    assert!(
        b.set_response(ActionSelection {
            prompt_id: 1,
            choice_id: 0
        })
        .is_err()
    );
}

#[test]
fn run_away_bypasses_move_and_ability_traps_but_does_not_allow_switching() {
    for trap in [
        "mean-look",
        "bind",
        "arena-trap",
        "shadow-tag",
        "magnet-pull",
    ] {
        for runaway in [false, true] {
            let mut user = mon("Runner", 1);
            user.types = vec![PokemonType::Steel];
            user.ability = runaway.then(|| "run-away".into());
            let mut target = mon("Foe", 100);
            match trap {
                "mean-look" => user.conditions.push(trap_condition(Rule::TrappedBy {
                    source: id(Side::Foes),
                })),
                "bind" => {
                    user.bound = Some(Bound {
                        by: Some(id(Side::Foes)),
                        turns_left: 4,
                        source: "bind".into(),
                    })
                }
                _ => target.ability = Some(trap.into()),
            }
            let mut b = game(vec![user, mon("Bench", 100)], vec![target], 255, true);
            b.set_active([vec![0], vec![0]]).unwrap();
            let p = prompt(&mut b);
            assert_eq!(p.choices.iter().any(flee), runaway, "{trap}");
            assert!(
                !p.choices.iter().any(|c| matches!(c, Choice::Switch { .. })),
                "{trap}"
            );
            if runaway {
                assert!(escaped(&attempt(&mut b)));
            }
        }
    }
}

#[test]
fn suppressed_run_away_uses_normal_escape_odds() {
    for gas in [false, true] {
        let mut user = mon("Runner", 1);
        user.ability = Some("run-away".into());
        user.ability_suppressed = !gas;
        let mut foe = mon("Foe", 500);
        foe.ability = gas.then(|| "neutralizing-gas".into());
        let mut b = game(vec![user], vec![foe], 255, true);
        let result = attempt(&mut b);
        assert!(!escaped(&result));
        assert!(result.events.contains(&BattleEvent::EscapeAttempt {
            actor: id(Side::Allies),
            success: false
        }));
        assert!(b.participants(Side::Allies)[0].hp < 1000);
    }
}

#[test]
fn ghost_escape_is_guaranteed_even_when_trapped_and_ability_is_suppressed() {
    let mut user = mon("Ghost", 1);
    user.types = vec![PokemonType::Ghost];
    user.ability_suppressed = true;
    user.conditions.push(trap_condition(Rule::Trapped));
    let mut b = game(vec![user], vec![mon("Foe", 500)], 255, true);
    assert!(escaped(&attempt(&mut b)));
}

#[test]
fn equal_or_higher_raw_speed_guarantees_escape_despite_paralysis_and_stages() {
    for speed in [100, 101] {
        let mut user = mon("Runner", speed);
        user.status = Some(Status::Paralyzed);
        user.stages.insert(Stat::Speed, -6);
        let mut b = game(vec![user], vec![mon("Foe", 100)], 255, true);
        assert!(escaped(&attempt(&mut b)));
    }
}

#[test]
fn failed_escape_consumes_turn_and_repeated_attempts_improve_odds() {
    // 50/100 * 128 = 64 on the first attempt, then 94 after one failure.
    let mut b = game(vec![mon("Runner", 50)], vec![mon("Foe", 100)], 64, true);
    let pp = b.participants(Side::Allies)[0].move_pp.clone();
    let first = attempt(&mut b);
    assert!(!escaped(&first));
    assert_eq!(b.turn(), 1);
    let hp = b.participants(Side::Allies)[0].hp;
    assert!(hp < 1000);
    assert_eq!(b.participants(Side::Allies)[0].move_pp, pp);
    assert!(escaped(&attempt(&mut b)));
    assert_eq!(b.turn(), 2);
    assert_eq!(b.participants(Side::Allies)[0].hp, hp);
}

#[test]
fn escape_random_roll_uses_a_strict_threshold() {
    for (roll, success) in [(63, true), (64, false)] {
        let mut b = game(vec![mon("Runner", 50)], vec![mon("Foe", 100)], roll, true);
        assert_eq!(escaped(&attempt(&mut b)), success);
    }
}

#[test]
fn runaway_works_while_asleep_frozen_or_recharging() {
    for mode in 0..3 {
        let mut user = mon("Runner", 1);
        user.ability = Some("run-away".into());
        user.status = match mode {
            0 => Some(Status::Asleep),
            1 => Some(Status::Frozen),
            _ => None,
        };
        user.status_turns = 3;
        user.recharging = mode == 2;
        let status = user.status;
        let mut b = game(vec![user], vec![mon("Foe", 500)], 255, true);
        assert!(escaped(&attempt(&mut b)));
        assert_eq!(b.participants(Side::Allies)[0].status, status);
        assert_eq!(b.participants(Side::Allies)[0].status_turns, 3);
    }
}

#[test]
fn recharge_can_still_continue_when_escape_is_offered() {
    let mut user = mon("Runner", 1);
    user.recharging = true;
    let mut b = game(vec![user], vec![mon("Foe", 500)], 255, true);
    let p = prompt(&mut b);
    assert!(p.choices.iter().any(flee));
    assert!(!p.choices.iter().any(movement));
    choose(&mut b, &p, |c| matches!(c, Choice::Continue { .. }));
    let p = prompt(&mut b);
    choose(&mut b, &p, movement);
    let result = b.advance().unwrap();
    assert!(
        result
            .events
            .iter()
            .any(|e| matches!(e, BattleEvent::Message(m) if m.contains("must recharge")))
    );
    assert!(!b.participants(Side::Allies)[0].recharging);
}

#[test]
fn escape_remains_available_during_charging_and_scripted_continuations() {
    for scripted in [false, true] {
        let mut user = mon("Runner", 1);
        user.moves = vec!["fly".into()];
        user.ability = Some("run-away".into());
        if scripted {
            user.script_continuation = Some(ScriptContinuation {
                move_id: "fly".into(),
                target: id(Side::Foes),
                turn: 1,
                total_turns: 2,
                target_policy: ContinuationTarget::SameTarget,
            });
        } else {
            user.locked_move = Some(LockedMove {
                move_id: "fly".into(),
                target: id(Side::Foes),
                turns: 1,
                max_turns: 2,
                charging: true,
            });
        }
        let mut b = game(vec![user], vec![mon("Foe", 500)], 255, true);
        assert!(escaped(&attempt(&mut b)));
    }
}

#[test]
fn zero_pp_keeps_struggle_as_an_alternative_to_escape() {
    let mut user = mon("Runner", 1);
    user.move_pp.insert("tackle".into(), 0);
    let mut b = game(vec![user], vec![mon("Foe", 500)], 255, true);
    let p = prompt(&mut b);
    assert!(p.choices.iter().any(flee));
    assert!(
        p.choices
            .iter()
            .any(|c| matches!(c, Choice::UseMove { move_id, .. } if move_id == "struggle"))
    );
}

#[test]
fn escape_is_not_a_replacement_choice_and_bench_runaway_does_not_help() {
    let mut fainted = mon("Fainted", 1);
    fainted.hp = 0;
    let mut bench = mon("Bench", 1);
    bench.ability = Some("run-away".into());
    let mut b = game(vec![fainted, bench], vec![mon("Foe", 500)], 255, true);
    b.set_active([vec![0], vec![0]]).unwrap();
    let p = prompt(&mut b);
    assert_eq!(p.reason, PromptReason::Replacement);
    assert!(p.choices.iter().all(|c| matches!(c, Choice::Switch { .. })));

    let mut reserve = mon("Bench", 1);
    reserve.ability = Some("run-away".into());
    let mut b = game(
        vec![mon("Runner", 1), reserve],
        vec![mon("Foe", 500)],
        255,
        true,
    );
    b.set_active([vec![0], vec![0]]).unwrap();
    assert!(!escaped(&attempt(&mut b)));
}

#[test]
fn escape_replaces_the_whole_partys_actions_in_multi_active_battles() {
    let mut second = mon("Second", 1);
    second.ability = Some("run-away".into());
    let mut b = game(
        vec![mon("First", 1), second],
        vec![mon("Foe", 500)],
        255,
        true,
    );
    let p = prompt(&mut b);
    choose(&mut b, &p, movement);
    let p = prompt(&mut b);
    assert_eq!(p.actor.index, 1);
    choose(&mut b, &p, flee);
    let p = prompt(&mut b);
    assert_eq!(p.actor.side, Side::Foes);
    choose(&mut b, &p, movement);
    let result = b.advance().unwrap();
    assert!(escaped(&result));
    assert!(
        !result
            .events
            .iter()
            .any(|e| matches!(e, BattleEvent::Damage { .. }))
    );
}

fn trap_condition(rule: Rule) -> Condition {
    Condition {
        name: "trap".into(),
        value: 1,
        turns_left: None,
        rules: vec![rule],
        group: None,
        end_message: None,
    }
}

#[test]
fn shed_shell_does_not_grant_escape_and_suppressed_runaway_cannot_bypass_traps() {
    for shell in [false, true] {
        let mut user = mon("Runner", 1000);
        user.ability = Some("run-away".into());
        user.ability_suppressed = true;
        user.item = shell.then(|| "shed-shell".into());
        user.conditions.push(trap_condition(Rule::Trapped));
        let mut b = game(
            vec![user, mon("Bench", 100)],
            vec![mon("Foe", 100)],
            0,
            true,
        );
        b.set_active([vec![0], vec![0]]).unwrap();
        let p = prompt(&mut b);
        assert!(!p.choices.iter().any(flee));
        assert_eq!(
            p.choices.iter().any(|c| matches!(c, Choice::Switch { .. })),
            shell
        );
    }
}

#[test]
fn failed_escape_ends_recharge_and_interrupted_charge_without_stranding_the_user() {
    for scripted in [false, true] {
        let mut user = mon("Runner", 1);
        user.moves = vec!["fly".into()];
        user.semi_invulnerable = true;
        if scripted {
            user.script_continuation = Some(ScriptContinuation {
                move_id: "fly".into(),
                target: id(Side::Foes),
                turn: 1,
                total_turns: 2,
                target_policy: ContinuationTarget::SameTarget,
            });
        } else {
            user.locked_move = Some(LockedMove {
                move_id: "fly".into(),
                target: id(Side::Foes),
                turns: 1,
                max_turns: 2,
                charging: true,
            });
        }
        let mut b = game(vec![user], vec![mon("Foe", 500)], 255, true);
        assert!(!escaped(&attempt(&mut b)));
        let user = &b.participants(Side::Allies)[0];
        assert!(!user.semi_invulnerable);
        assert!(user.locked_move.is_none());
        assert!(user.script_continuation.is_none());
        assert!(user.hp < 1000);
    }
    let mut user = mon("Runner", 1);
    user.recharging = true;
    let mut b = game(vec![user], vec![mon("Foe", 500)], 255, true);
    assert!(!escaped(&attempt(&mut b)));
    assert!(!b.participants(Side::Allies)[0].recharging);
}

#[test]
fn failed_party_escape_consumes_other_allied_actions_and_ticks_their_statuses() {
    let mut first = mon("First", 1);
    first.status = Some(Status::Poisoned);
    let mut second = mon("Second", 1);
    second.status = Some(Status::Poisoned);
    let mut b = game(vec![first, second], vec![mon("Foe", 500)], 255, true);
    let first_pp = b.participants(Side::Allies)[0].move_pp.clone();
    let second_pp = b.participants(Side::Allies)[1].move_pp.clone();
    let result = attempt(&mut b);
    assert!(!escaped(&result));
    for (p, pp) in b
        .participants(Side::Allies)
        .iter()
        .zip([first_pp, second_pp])
    {
        assert!(p.hp < 1000);
        assert!(p.acted);
        assert_eq!(p.move_pp, pp);
    }
    assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
}
