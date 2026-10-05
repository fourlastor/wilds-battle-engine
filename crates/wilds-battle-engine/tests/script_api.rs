//! What scripted moves can read and do, one small Lua move at a time.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, Gender, HiddenKind,
    MoveCatalog, Pokemon, PokemonType, Side, Stat, Status, WeatherKind,
};

/// Dice that always give the same number: `chance(p)` succeeds when `p` is at least that number.
struct FixedRng(f32);
impl BattleRng for FixedRng {
    fn next_u64(&mut self) -> u64 {
        0
    }
    fn fraction(&mut self) -> f32 {
        self.0
    }
}

const HEAD: &str = "type=Type.Normal, category=Category.Physical, pp=40";
const WAIT: &str = "{id='wait', name='Wait', type=Type.Normal, category=Category.Status, pp=40, \
    target=Target.User, script=function(_) end}";

fn mon(name: &str, moves: &[&str]) -> Pokemon {
    let mut pokemon = Pokemon::new(name, moves.iter().map(|id| id.to_string()).collect());
    pokemon.hp = 400;
    pokemon.max_hp = 400;
    pokemon
}

struct Game {
    battle: Battle,
}

impl Game {
    fn with_dice(dice: f32, moves: &[&str], allies: Vec<Pokemon>, foes: Vec<Pokemon>) -> Self {
        let source = format!("return {{{WAIT}, {}}}", moves.join(", "));
        let catalog = MoveCatalog::from_lua(&source).unwrap();
        Self {
            battle: Battle::with_rng(FixedRng(dice), [allies, foes], catalog).unwrap(),
        }
    }
    /// A battle where 50% chances succeed, nothing misses and nothing is a critical hit.
    fn new(moves: &[&str], allies: Vec<Pokemon>, foes: Vec<Pokemon>) -> Self {
        Self::with_dice(0.5, moves, allies, foes)
    }
    /// One ally with these moves against one foe that only waits.
    fn duel(moves: &[&str], ids: &[&str]) -> Self {
        Self::new(moves, vec![mon("Ally", ids)], vec![mon("Foe", &["wait"])])
    }

    /// Plays a turn. Each Pokémon that is asked takes the next pick: a move id, or `id>2` to aim
    /// at the second Pokémon of the side the move is used on.
    fn events(&mut self, picks: &[&str]) -> Vec<BattleEvent> {
        let mut picks = picks.iter();
        loop {
            let result = self.battle.advance().unwrap();
            if !result.events.is_empty() {
                return result.events;
            }
            let AdvanceStatus::Awaiting(prompt) = result.status else {
                return result.events;
            };
            let pick = picks
                .next()
                .expect("a pick for every Pokémon that is asked");
            let (id, slot) = match pick.split_once('>') {
                Some((id, slot)) => (id, Some(slot.parse::<usize>().unwrap() - 1)),
                None => (*pick, None),
            };
            let choice = prompt
                .choices
                .iter()
                .find(|choice| {
                    let Choice::UseMove {
                        move_id, target, ..
                    } = choice;
                    move_id == id && slot.is_none_or(|slot| target.index == slot)
                })
                .unwrap_or_else(|| panic!("{pick} is not on offer: {:?}", prompt.choices));
            self.battle
                .set_response(ActionSelection {
                    prompt_id: prompt.id,
                    choice_id: choice.id(),
                })
                .unwrap();
        }
    }
    fn turn(&mut self, picks: &[&str]) -> Vec<String> {
        self.events(picks)
            .into_iter()
            .filter_map(|event| match event {
                BattleEvent::Message(text) => Some(text),
                _ => None,
            })
            .collect()
    }
    fn ally(&self, index: usize) -> &Pokemon {
        &self.battle.participants(Side::Allies)[index]
    }
    fn foe(&self, index: usize) -> &Pokemon {
        &self.battle.participants(Side::Foes)[index]
    }
    /// HP the first foe has lost.
    fn dealt(&self) -> u16 {
        self.foe(0).max_hp - self.foe(0).hp
    }
}

fn said(messages: &[String], text: &str) -> bool {
    messages.iter().any(|message| message.contains(text))
}

/// One scripted move called `test`.
fn test_move(extra: &str, body: &str) -> String {
    format!("{{id='test', name='Test', {HEAD}, {extra} script=function(ctx) {body} end}}")
}

/// Damage the first foe takes from a one-turn script, with the given dice.
fn damage_of(dice: f32, extra: &str, body: &str) -> u16 {
    let mut game = Game::with_dice(
        dice,
        &[&test_move(extra, body)],
        vec![mon("Ally", &["test"])],
        vec![mon("Foe", &["wait"])],
    );
    game.turn(&["test", "wait"]);
    game.dealt()
}

// ---------------------------------------------------------------------------
// Status, stats and other things only plain moves could do before

#[test]
fn scripts_inflict_and_cure_status() {
    let script = test_move(
        "",
        "if ctx.target.status == nil then
           ctx:message(tostring(ctx:apply_status(ctx.target, ctx.Status.Burned)))
           ctx:message(tostring(ctx.target.status == ctx.Status.Burned))
         else
           ctx:message(tostring(ctx:apply_status(ctx.target, ctx.Status.Paralyzed, {announce_failure=true})))
           ctx:message(tostring(ctx:cure_status(ctx.target, ctx.Status.Paralyzed)))
           ctx:message(tostring(ctx:cure_status(ctx.target)))
         end",
    );
    let mut game = Game::duel(&[&script], &["test"]);
    let first = game.turn(&["test", "wait"]);
    assert!(said(&first, "Foe was burned!"));
    assert_eq!(first.iter().filter(|line| *line == "true").count(), 2);
    assert_eq!(game.foe(0).status, Some(Status::Burned));
    let second = game.turn(&["test", "wait"]);
    // A second status does not replace the first; curing needs the right status, or none named.
    assert!(said(&second, "But it failed!"));
    assert_eq!(
        second
            .iter()
            .filter(|line| *line == "true" || *line == "false")
            .collect::<Vec<_>>(),
        ["false", "false", "true"]
    );
    assert_eq!(game.foe(0).status, None);
}

#[test]
fn status_respects_type_immunity_chance_and_replace() {
    let script = test_move(
        "",
        "ctx:message('poison ' .. tostring(ctx:apply_status(ctx.target, ctx.Status.Poisoned)))
         ctx:message('rare ' .. tostring(ctx:apply_status(ctx.target, ctx.Status.Burned, {chance=0.3})))
         ctx:message('likely ' .. tostring(ctx:apply_status(ctx.target, ctx.Status.Burned, {chance=0.6})))
         ctx:message('over ' .. tostring(ctx:apply_status(ctx.target, ctx.Status.Asleep, {replace=true})))",
    );
    let mut foe = mon("Foe", &["wait"]);
    foe.types = vec![PokemonType::Steel];
    let mut game = Game::new(&[&script], vec![mon("Ally", &["test"])], vec![foe]);
    let lines = game.turn(&["test", "wait"]);
    assert!(said(&lines, "poison false"));
    assert!(said(&lines, "rare false"));
    assert!(said(&lines, "likely true"));
    assert!(said(&lines, "over true"));
    assert_eq!(game.foe(0).status, Some(Status::Asleep));
}

#[test]
fn scripts_change_any_stat_and_learn_how_far_it_moved() {
    let script = test_move(
        "",
        "ctx:message('down ' .. ctx:change_stat(ctx.target, ctx.Stat.Defense, -2))
         ctx:message('up ' .. ctx:change_stat(ctx.user, ctx.Stat.Attack, 6))
         ctx:message('capped ' .. ctx:change_stat(ctx.user, ctx.Stat.Attack, 1))
         ctx:message('unlikely ' .. ctx:change_stat(ctx.target, ctx.Stat.Speed, -1, {chance=0.1}))
         ctx:message('stage ' .. ctx.target:stage(ctx.Stat.Defense) .. ' ' .. ctx.user.stages[ctx.Stat.Attack])
         ctx:reset_stats(ctx.user)
         ctx:message('after ' .. ctx.user:stage(ctx.Stat.Attack))",
    );
    let mut game = Game::duel(&[&script], &["test"]);
    let lines = game.turn(&["test", "wait"]);
    for expected in [
        "down -2",
        "up 6",
        "capped 0",
        "unlikely 0",
        "stage -2 6",
        "after 0",
    ] {
        assert!(said(&lines, expected), "{expected} in {lines:?}");
    }
    assert_eq!(game.foe(0).stage(Stat::Defense), -2);
    assert_eq!(game.ally(0).stage(Stat::Attack), 0);
}

#[test]
fn scripts_confuse_flinch_heal_and_hurt_anyone() {
    let script = test_move(
        "",
        "ctx:message('confused ' .. tostring(ctx:confuse(ctx.target)))
         ctx:message('again ' .. tostring(ctx:confuse(ctx.target)))
         ctx:message('flinch ' .. tostring(ctx:flinch(ctx.target, {chance=0.5})))
         ctx:message('lost ' .. ctx:hurt(ctx.target, {fraction=1/4}))
         ctx:message('lost ' .. ctx:hurt(ctx.user, 30))
         ctx:message('back ' .. ctx:heal(ctx.target, {fraction=1/8}))
         ctx:message('back ' .. ctx:heal(ctx.user, 1000))",
    );
    let mut game = Game::duel(&[&script], &["test"]);
    let lines = game.turn(&["test", "wait"]);
    for expected in [
        "confused true",
        "again false",
        "flinch true",
        "lost 100",
        "lost 30",
        "back 50",
        "back 30",
    ] {
        assert!(said(&lines, expected), "{expected} in {lines:?}");
    }
    assert!(said(&lines, "Foe flinched!"));
    assert_eq!(game.foe(0).hp, 350);
    assert_eq!(game.ally(0).hp, 400);
}

#[test]
fn try_hit_runs_the_usual_checks_for_moves_without_damage() {
    let moves = [
        format!(
            "{{id='guard', name='Guard', {HEAD}, target=Target.User, priority=4,
               script=function(ctx) ctx:protect() end}}"
        ),
        format!(
            "{{id='shaky', name='Shaky', {HEAD}, accuracy=0.4,
               script=function(ctx) if ctx:try_hit() then ctx:apply_status(ctx.target, ctx.Status.Asleep) end end}}"
        ),
        format!(
            "{{id='sure', name='Sure', {HEAD}, accuracy=0.4,
               script=function(ctx)
                 if ctx:try_hit(nil, {{never_miss=true, ignore_protect=true}}) then ctx:message('through') end
               end}}"
        ),
    ];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut game = Game::new(
        &moves,
        vec![mon("Ally", &["shaky", "sure"])],
        vec![mon("Foe", &["wait", "guard"])],
    );
    // 40% accuracy against dice of 0.5: a miss, and the move counts as failed.
    assert!(said(
        &game.turn(&["shaky", "wait"]),
        "Ally's attack missed!"
    ));
    assert!(game.ally(0).last_move_failed);
    assert!(said(
        &game.turn(&["shaky", "guard"]),
        "Foe protected itself!"
    ));
    assert!(said(&game.turn(&["sure", "guard"]), "through"));
    assert!(!game.ally(0).last_move_failed);
}

#[test]
fn protecting_oneself_turn_after_turn_works_less_often() {
    let guard = format!(
        "{{id='guard', name='Guard', {HEAD}, target=Target.User, priority=4,
           script=function(ctx) ctx:message('held ' .. tostring(ctx:protect())) end}}"
    );
    let cover = format!(
        "{{id='cover', name='Cover', {HEAD}, target=Target.Ally, priority=4,
           script=function(ctx) ctx:message('covered ' .. tostring(ctx:protect(ctx.target))) end}}"
    );
    let mut game = Game::duel(&[&guard], &["guard"]);
    assert!(said(&game.turn(&["guard", "wait"]), "held true"));
    // The second use in a row has one chance in three, which dice of 0.5 do not give.
    let second = game.turn(&["guard", "wait"]);
    assert!(said(&second, "But it failed!") && said(&second, "held false"));
    assert!(game.ally(0).last_move_failed);
    // Failing breaks the streak.
    assert!(said(&game.turn(&["guard", "wait"]), "held true"));

    // Protecting someone else is not held to that.
    let mut game = Game::new(
        &[&cover],
        vec![mon("Ally", &["wait"]), mon("Partner", &["cover"])],
        vec![mon("Foe", &["wait"])],
    );
    for _ in 0..3 {
        assert!(said(&game.turn(&["wait", "cover", "wait"]), "covered true"));
    }
}

#[test]
fn a_hit_follows_the_check_made_just_before_it() {
    let run = |dice: f32, body: &str| {
        let mut game = Game::with_dice(
            dice,
            &[&test_move("accuracy=0.4,", body)],
            vec![mon("Ally", &["test"])],
            vec![mon("Foe", &["wait"])],
        );
        let lines = game.turn(&["test", "wait"]);
        (lines, game.dealt())
    };
    // The check missed: the hit that follows does not land, and says nothing more.
    let (lines, dealt) = run(
        0.5,
        "ctx:try_hit()
         local hit = ctx:damage(40)
         ctx:message('missed ' .. tostring(hit.missed) .. ' reached ' .. tostring(ctx:reached()))",
    );
    assert_eq!(dealt, 0);
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.contains("attack missed"))
            .count(),
        1
    );
    assert!(said(&lines, "missed true reached false"));
    // The check passed: the hit lands without rolling again. The hit after that checks for itself.
    let (lines, dealt) = run(
        0.5,
        "ctx:try_hit(nil, {never_miss=true})
         ctx:message('first ' .. tostring(ctx:damage(40).hit))
         ctx:message('second ' .. tostring(ctx:damage(40).hit))",
    );
    assert!(said(&lines, "first true") && said(&lines, "second false"));
    assert!(dealt > 0);
}

#[test]
fn power_and_hp_amounts_drop_their_fraction() {
    let whole = damage_of(0.5, "", "ctx:damage(100)");
    assert_eq!(damage_of(0.5, "", "ctx:damage(150 * 2 / 3)"), whole);
    assert_eq!(damage_of(0.5, "", "ctx:damage(100.9)"), whole);
    assert_eq!(
        damage_of(0.5, "", "ctx:direct_damage(ctx.target, 25 / 2)"),
        12
    );
    assert_eq!(damage_of(0.5, "", "ctx:hurt(ctx.target, 99 / 2)"), 49);
}

// ---------------------------------------------------------------------------
// Damage options

#[test]
fn damage_can_use_another_type() {
    let steel_foe = |body: &str| {
        let mut foe = mon("Foe", &["wait"]);
        foe.types = vec![PokemonType::Steel];
        let mut game = Game::new(
            &[&test_move("", body)],
            vec![mon("Ally", &["test"])],
            vec![foe],
        );
        let lines = game.turn(&["test", "wait"]);
        (game.dealt(), lines)
    };
    let (normal, _) = steel_foe("ctx:damage(60)");
    let (fire, lines) = steel_foe("ctx:damage(60, {type=ctx.Type.Fire})");
    assert!(said(&lines, "It's super effective!"));
    // Resisted with the user's own type bonus, against super effective without it.
    assert!(fire > normal * 2, "{fire} vs {normal}");
    let (both, _) = steel_foe("ctx:damage(60, {type=ctx.Type.Fighting, also_type=ctx.Type.Fire})");
    assert!(
        both > fire,
        "two super effective types multiply: {both} vs {fire}"
    );
    let (special_case, lines) =
        steel_foe("ctx:damage(60, {type=ctx.Type.Ice, effective={[ctx.Type.Steel]=2}})");
    assert!(said(&lines, "It's super effective!"));
    assert_eq!(special_case, fire);
}

#[test]
fn damage_can_use_other_stats() {
    let lopsided = |body: &str| {
        let mut ally = mon("Ally", &["test"]);
        ally.attack = 50;
        ally.sp_attack = 200;
        let mut foe = mon("Foe", &["wait"]);
        foe.attack = 300;
        foe.sp_defense = 200;
        let mut game = Game::new(&[&test_move("", body)], vec![ally], vec![foe]);
        game.turn(&["test", "wait"]);
        game.dealt()
    };
    let plain = lopsided("ctx:damage(60)");
    let special = lopsided("ctx:damage(60, {category=ctx.Category.Special})");
    let psyshock =
        lopsided("ctx:damage(60, {category=ctx.Category.Special, defense_stat=ctx.Stat.Defense})");
    let foul_play = lopsided("ctx:damage(60, {attack_from=ctx.target})");
    let body_press = lopsided("ctx:damage(60, {attack_stat=ctx.Stat.SpAttack})");
    assert!(
        special > plain,
        "Sp. Atk 200 against Sp. Def 200 beats Attack 50 against Defense 100"
    );
    assert!(
        psyshock > special * 3 / 2,
        "the same Sp. Atk against Defense 100: {psyshock} vs {special}"
    );
    assert!(
        foul_play > plain * 5,
        "the foe's Attack of 300: {foul_play} vs {plain}"
    );
    assert_eq!(body_press, psyshock);
}

#[test]
fn damage_can_ignore_the_targets_stat_changes() {
    let boosted = |body: &str| {
        let mut game = Game::duel(&[&test_move("", body)], &["test"]);
        game.turn(&["test", "wait"]);
        game.dealt()
    };
    let raise = "ctx:change_stat(ctx.target, ctx.Stat.Defense, 6)";
    let through = boosted(&format!("{raise} ctx:damage(60, {{ignore_stages=true}})"));
    let blunted = boosted(&format!("{raise} ctx:damage(60)"));
    let untouched = boosted("ctx:damage(60)");
    assert_eq!(through, untouched);
    assert!(blunted * 3 < untouched);
}

#[test]
fn damage_reports_critical_hits_recoil_and_knockouts() {
    let script = test_move(
        "",
        "local hit = ctx:damage(60, {always_crit=true, recoil=0.5})
         ctx:message('critical ' .. tostring(hit.critical) .. ' fainted ' .. tostring(hit.fainted))
         local last = ctx:damage(250)
         ctx:message('then ' .. tostring(last.fainted) .. ' ' .. last.effectiveness)",
    );
    let mut foe = mon("Foe", &["wait"]);
    foe.hp = 150;
    let mut game = Game::new(&[&script], vec![mon("Ally", &["test"])], vec![foe]);
    let lines = game.turn(&["test", "wait"]);
    assert!(said(&lines, "A critical hit!"));
    assert!(said(&lines, "critical true fainted false"));
    assert!(said(&lines, "Ally was damaged by the recoil!"));
    assert!(said(&lines, "then true 1"));
    assert!(game.ally(0).hp < 400);
}

#[test]
fn high_crit_raises_the_odds_of_a_critical_hit() {
    // Dice of 0.1 beat one chance in eight, but not one in twenty-four.
    let plain = damage_of(0.1, "", "ctx:damage(60)");
    let keen = damage_of(0.1, "", "ctx:damage(60, {high_crit=true})");
    assert!(keen > plain);
}

#[test]
fn damage_can_skip_accuracy_and_protection() {
    let guard = format!(
        "{{id='guard', name='Guard', {HEAD}, target=Target.User, priority=4,
           script=function(ctx) ctx:protect() end}}"
    );
    let run = |options: &str, foe_move: &str| {
        let strike = test_move("accuracy=0.3,", &format!("ctx:damage(60, {options})"));
        let mut game = Game::new(
            &[&guard, &strike],
            vec![mon("Ally", &["test"])],
            vec![mon("Foe", &["wait", "guard"])],
        );
        game.turn(&["test", foe_move]);
        game.dealt()
    };
    assert_eq!(run("{}", "wait"), 0, "30% accuracy misses with dice of 0.5");
    assert!(run("{never_miss=true}", "wait") > 0);
    assert_eq!(run("{never_miss=true}", "guard"), 0);
    assert!(run("{never_miss=true, ignore_protect=true}", "guard") > 0);
}

#[test]
fn feint_lifts_protection_for_the_rest_of_the_turn() {
    let guard = format!(
        "{{id='guard', name='Guard', {HEAD}, target=Target.User, priority=4,
           script=function(ctx) ctx:protect() end}}"
    );
    let feint = test_move(
        "",
        "ctx:message('lifted ' .. tostring(ctx:break_protect()))
         ctx:damage(30)",
    );
    let mut game = Game::new(
        &[&guard, &feint],
        vec![mon("Ally", &["test"])],
        vec![mon("Foe", &["guard"])],
    );
    let lines = game.turn(&["test", "guard"]);
    assert!(said(&lines, "lifted true"));
    assert!(game.dealt() > 0);
    assert!(!game.foe(0).protected);
}

#[test]
fn scripted_multi_hit_and_exact_damage() {
    let script = test_move(
        "",
        "local flurry = ctx:multi_hit(20, 3, 3)
         ctx:message('hits ' .. flurry.hits .. ' damage ' .. flurry.damage)
         local exact = ctx:direct_damage(ctx.target, 40)
         ctx:message('exact ' .. exact.damage)
         local spared = ctx:direct_damage(ctx.target, 9999, {min_target_hp=1})
         ctx:message('left ' .. ctx.target.hp)",
    );
    let mut game = Game::duel(&[&script], &["test"]);
    let lines = game.turn(&["test", "wait"]);
    assert!(said(&lines, "Hit 3 times!"));
    assert!(said(&lines, "hits 3 damage "));
    assert!(said(&lines, "exact 40"));
    assert!(said(&lines, "left 1"));
    assert_eq!(game.foe(0).hp, 1);
}

#[test]
fn exact_damage_respects_immunity_unless_typeless() {
    let run = |options: &str| {
        let script = test_move("", &format!("ctx:direct_damage(ctx.target, 40{options})"));
        let mut foe = mon("Foe", &["wait"]);
        foe.types = vec![PokemonType::Ghost];
        let mut game = Game::new(&[&script], vec![mon("Ally", &["test"])], vec![foe]);
        let lines = game.turn(&["test", "wait"]);
        (game.dealt(), said(&lines, "It doesn't affect Foe"))
    };
    assert_eq!(run(""), (0, true));
    assert_eq!(run(", {typeless=true}"), (40, false));
}

#[test]
fn what_a_script_reads_is_always_current() {
    let script = test_move(
        "",
        "local before = ctx.target.hp
         local hit = ctx:damage(60)
         ctx:message('lost ' .. tostring(before - ctx.target.hp == hit.damage))
         ctx:apply_status(ctx.user, ctx.Status.Paralyzed)
         ctx:message('now ' .. tostring(ctx.user.status == ctx.Status.Paralyzed))",
    );
    let mut game = Game::duel(&[&script], &["test"]);
    let lines = game.turn(&["test", "wait"]);
    assert!(said(&lines, "lost true"));
    assert!(said(&lines, "now true"));
}

// ---------------------------------------------------------------------------
// Hiding, and the moves that still reach

fn fly() -> String {
    format!(
        "{{id='fly', name='Fly', {HEAD},
           script=function(ctx)
             if ctx.turn == 1 then
               ctx:force_move(2, ctx.TargetPolicy.SameTarget)
               ctx:hide(ctx.Hidden.Air)
               return
             end
             ctx:damage(90)
           end}}"
    )
}

#[test]
fn a_hidden_pokemon_is_only_reached_by_moves_that_say_so() {
    let gust = format!(
        "{{id='gust', name='Gust', {HEAD}, hits_hidden={{Hidden.Air}},
           script=function(ctx)
             local power = 40
             if ctx.target.hidden == ctx.Hidden.Air then power = power * 2 end
             ctx:damage(power)
           end}}"
    );
    let quake = format!(
        "{{id='quake', name='Quake', {HEAD},
           script=function(ctx) ctx:damage(40, {{hits_hidden={{ctx.Hidden.Underground}}}}) end}}"
    );
    let tackle = format!(
        "{{id='tackle', name='Tackle', {HEAD}, effects={{{{kind=Effect.Damage, power=40}}}}}}"
    );
    let plain_gust = format!(
        "{{id='breeze', name='Breeze', {HEAD}, hits_hidden={{Hidden.Air}}, effects={{{{kind=Effect.Damage, power=40}}}}}}"
    );
    let moves = [fly(), gust, quake, tackle, plain_gust];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let lost_while_up = |foe_move: &str| {
        let mut ally = mon("Ally", &["fly"]);
        ally.speed = 200;
        let mut game = Game::new(&moves, vec![ally], vec![mon("Foe", &[foe_move])]);
        let lines = game.turn(&["fly", foe_move]);
        assert_eq!(game.ally(0).hidden(), Some(HiddenKind::Air));
        (
            400 - game.ally(0).hp,
            said(&lines, "Ally avoided the attack!"),
        )
    };
    assert_eq!(lost_while_up("tackle"), (0, true));
    assert_eq!(lost_while_up("quake"), (0, true));
    let (breeze, _) = lost_while_up("breeze");
    let (gust, avoided) = lost_while_up("gust");
    assert!(!avoided);
    assert!(breeze > 0);
    assert!(
        gust > breeze * 3 / 2,
        "double power in the air: {gust} vs {breeze}"
    );
}

#[test]
fn a_hidden_pokemon_comes_back_when_it_acts_and_can_be_knocked_down() {
    let smack = format!(
        "{{id='smack', name='Smack', {HEAD}, hits_hidden={{Hidden.Air}},
           script=function(ctx)
             ctx:damage(20)
             ctx:message('down ' .. tostring(ctx:unhide(ctx.target)))
           end}}"
    );
    let moves = [fly(), smack];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut ally = mon("Ally", &["fly"]);
    ally.speed = 200;
    let mut game = Game::new(&moves, vec![ally], vec![mon("Foe", &["wait", "smack"])]);
    game.turn(&["fly", "wait"]);
    assert!(game.ally(0).hidden().is_some());
    // The second turn is forced: only the foe is asked.
    game.turn(&["wait"]);
    assert!(game.ally(0).hidden().is_none());
    assert!(game.dealt() > 0);

    // Knocked down on the turn it flew up: the move is called off and it chooses again.
    let before = game.dealt();
    let lines = game.turn(&["fly", "smack"]);
    assert!(said(&lines, "down true"));
    assert!(game.ally(0).hidden().is_none());
    assert!(game.ally(0).script_continuation.is_none());
    game.turn(&["fly", "wait"]);
    assert_eq!(game.dealt(), before);
}

// ---------------------------------------------------------------------------
// Weather

#[test]
fn rain_and_hail_exist() {
    let rain = format!(
        "{{id='rain', name='Rain', {HEAD}, target=Target.Field,
           script=function(ctx)
             ctx:message('started ' .. tostring(ctx:set_weather(ctx.Weather.Rain, 2)))
             ctx:message('raining ' .. tostring(ctx.weather == ctx.Weather.Rain))
           end}}"
    );
    let hail = format!(
        "{{id='hail', name='Hail', {HEAD}, target=Target.Field, effects={{{{kind=Effect.Weather, weather=Weather.Hail, turns=2}}}}}}"
    );
    let squirt = "{id='squirt', name='Squirt', type=Type.Water, category=Category.Special, pp=40,
           script=function(ctx) ctx:damage(40) end}"
        .to_string();
    let clear = format!(
        "{{id='clear', name='Clear', {HEAD}, target=Target.Field,
           script=function(ctx) ctx:message('cleared ' .. tostring(ctx:clear_weather())) end}}"
    );
    let moves = [rain, hail, squirt, clear];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut ice = mon("Ice", &["wait"]);
    ice.types = vec![PokemonType::Ice];
    let mut game = Game::new(
        &moves,
        vec![mon("Ally", &["rain", "hail", "squirt", "clear"])],
        vec![mon("Foe", &["wait"]), ice],
    );
    game.turn(&["squirt", "wait", "wait"]);
    let dry = game.dealt();
    let lines = game.turn(&["rain", "wait", "wait"]);
    assert!(said(&lines, "It started to rain!"));
    assert!(said(&lines, "started true"));
    assert!(said(&lines, "raining true"));
    assert!(said(&lines, "Rain continues to fall."));
    assert_eq!(game.battle.weather().unwrap().kind, WeatherKind::Rain);
    game.turn(&["squirt", "wait", "wait"]);
    let wet = game.dealt() - dry;
    assert!(
        wet > dry * 14 / 10,
        "water moves are half as strong again: {wet} vs {dry}"
    );

    assert!(said(&game.turn(&["clear", "wait", "wait"]), "cleared true"));
    assert!(game.battle.weather().is_none());
    let before = (game.ally(0).hp, game.foe(0).hp, game.foe(1).hp);
    let lines = game.turn(&["hail", "wait", "wait"]);
    assert!(said(&lines, "It started to hail!"));
    assert!(said(&lines, "Ally is buffeted by the hail!"));
    assert!(!said(&lines, "Ice is buffeted"));
    assert_eq!(
        (game.ally(0).hp, game.foe(0).hp, game.foe(1).hp),
        (before.0 - 25, before.1 - 25, before.2)
    );
}

// ---------------------------------------------------------------------------
// Marks: what one move leaves for another

#[test]
fn a_mark_left_by_one_move_is_read_by_another() {
    let shrink = format!(
        "{{id='shrink', name='Shrink', {HEAD}, target=Target.User,
           script=function(ctx) ctx:change_stat(ctx.user, ctx.Stat.Evasion, 2) ctx:mark(ctx.user, 'minimized') end}}"
    );
    let stomp = format!(
        "{{id='stomp', name='Stomp', {HEAD}, accuracy=Accuracy.Always,
           script=function(ctx)
             local power = 65
             if ctx.target.marks.minimized then power = power * 2 end
             ctx:damage(power)
           end}}"
    );
    let moves = [shrink, stomp];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut game = Game::new(
        &moves,
        vec![mon("Ally", &["stomp"])],
        vec![mon("Foe", &["wait", "shrink"])],
    );
    game.turn(&["stomp", "wait"]);
    let normal = game.dealt();
    game.turn(&["stomp", "shrink"]);
    let same_turn = game.dealt() - normal;
    assert_eq!(same_turn, normal, "the foe shrinks after being hit");
    game.turn(&["stomp", "wait"]);
    let doubled = game.dealt() - normal * 2;
    assert!(doubled > normal * 19 / 10, "{doubled} vs {normal}");
}

#[test]
fn marks_count_and_expire() {
    let cutter = test_move(
        "",
        "local chain = ctx.user.marks.cutter or 0
         local hit = ctx:damage(10 * 2 ^ chain)
         ctx:message('chain ' .. chain)
         if hit.hit then ctx:mark(ctx.user, 'cutter', math.min(chain + 1, 4), 2) else ctx:unmark(ctx.user, 'cutter') end",
    );
    let mut game = Game::duel(&[&cutter], &["test", "wait"]);
    assert!(said(&game.turn(&["test", "wait"]), "chain 0"));
    assert!(said(&game.turn(&["test", "wait"]), "chain 1"));
    assert!(said(&game.turn(&["test", "wait"]), "chain 2"));
    assert_eq!(game.ally(0).condition("cutter").unwrap().value, 3);
    // The mark lasts through the next turn only: one turn off and the chain is gone.
    game.turn(&["wait", "wait"]);
    assert!(game.ally(0).condition("cutter").is_none());
    assert!(said(&game.turn(&["test", "wait"]), "chain 0"));
}

#[test]
fn marks_on_a_side_and_on_the_field() {
    let script = test_move(
        "",
        "ctx:message('echo ' .. (ctx.field.marks.echo or 0) .. ' side ' .. tostring(ctx.user_side.marks.pledge) .. ' ' .. tostring(ctx.foe_side.marks.pledge))
         ctx:mark(ctx.field, 'echo', (ctx.field.marks.echo or 0) + 1, 2)
         ctx:mark(ctx.user.team, 'pledge', 1, 1)",
    );
    let mut game = Game::new(
        &[&script],
        vec![mon("Ally", &["test"])],
        vec![mon("Foe", &["test"])],
    );
    let lines = game.turn(&["test", "test"]);
    // Equal speed: the tie goes to the first Pokémon with these dice.
    assert!(said(&lines, "echo 0 side nil nil"));
    assert!(said(&lines, "echo 1 side nil 1"));
    let lines = game.turn(&["test", "test"]);
    assert!(said(&lines, "echo 2 side nil nil"));
    assert_eq!(game.battle.field_conditions()[0].value, 4);
}

// ---------------------------------------------------------------------------
// Timed effects and their rules

fn reflect() -> String {
    format!(
        "{{id='reflect', name='Reflect', {HEAD}, target=Target.User,
           script=function(ctx)
             local started = ctx:start_effect(ctx.user.team, 'reflect', {{turns=3, end_message='The wall wore off!',
               rules={{{{kind=ctx.Rule.DamageTaken, category=ctx.Category.Physical, factor=0.5, not_on_crit=true}}}}}})
             if not started then return ctx:fail() end
           end}}"
    )
}

#[test]
fn a_screen_halves_damage_for_a_few_turns_and_can_be_broken() {
    let tackle = format!(
        "{{id='tackle', name='Tackle', {HEAD}, effects={{{{kind=Effect.Damage, power=40}}}}}}"
    );
    let brick = format!(
        "{{id='brick', name='Brick', {HEAD},
           script=function(ctx)
             if ctx:end_effect(ctx.target.team, 'reflect') then ctx:message('It shattered the wall!') end
             ctx:damage(40)
           end}}"
    );
    let moves = [reflect(), tackle, brick];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let fresh = || {
        let mut foe = mon("Foe", &["reflect", "wait"]);
        foe.speed = 200;
        Game::new(&moves, vec![mon("Ally", &["tackle", "brick"])], vec![foe])
    };
    let mut game = fresh();
    game.turn(&["tackle", "wait"]);
    let open = game.dealt();
    game.turn(&["tackle", "reflect"]);
    let walled = game.dealt() - open;
    assert!(walled * 2 <= open + 1 && walled > 0, "{walled} vs {open}");
    assert_eq!(
        game.battle.side_conditions(Side::Foes)[0].turns_left,
        Some(2)
    );
    assert!(said(&game.turn(&["tackle", "reflect"]), "But it failed!"));
    let lines = game.turn(&["tackle", "wait"]);
    assert!(said(&lines, "The wall wore off!"));
    assert!(game.battle.side_conditions(Side::Foes).is_empty());

    let mut game = fresh();
    game.turn(&["tackle", "reflect"]);
    let walled = game.dealt();
    let lines = game.turn(&["brick", "wait"]);
    assert!(said(&lines, "It shattered the wall!"));
    assert!(game.dealt() - walled > walled * 3 / 2);
}

#[test]
fn effects_can_hurt_drain_and_heal_each_turn() {
    let script = test_move(
        "",
        "ctx:start_effect(ctx.foe_side, 'fire_sea', {turns=2, rules={{kind=ctx.Rule.DamageEachTurn, fraction=1/8,
           except_types={ctx.Type.Fire}, message='{name} is hurt by the sea of fire!'}}})
         ctx:start_effect(ctx.target, 'seeded', {rules={{kind=ctx.Rule.DrainEachTurn, fraction=1/8, to=ctx.user,
           message=\"{name}'s health is sapped!\"}}})
         ctx:start_effect(ctx.user, 'rooted', {rules={{kind=ctx.Rule.HealEachTurn, fraction=1/16}}})",
    );
    let mut fire = mon("Fire", &["wait"]);
    fire.types = vec![PokemonType::Fire];
    let mut ally = mon("Ally", &["test", "wait"]);
    ally.hp = 100;
    let mut game = Game::new(&[&script], vec![ally], vec![mon("Foe", &["wait"]), fire]);
    let lines = game.turn(&["test", "wait", "wait"]);
    assert!(said(&lines, "Foe is hurt by the sea of fire!"));
    assert!(!said(&lines, "Fire is hurt"));
    assert!(said(&lines, "Foe's health is sapped!"));
    // The foe loses 50 to the fire and 50 to the seed; the user gets the 50 and 25 from its roots.
    assert_eq!(game.foe(0).hp, 300);
    assert_eq!(game.foe(1).hp, 400);
    assert_eq!(game.ally(0).hp, 175);
    game.turn(&["wait", "wait", "wait"]);
    assert_eq!(game.foe(0).hp, 200);
    assert_eq!(game.ally(0).hp, 250);
    // The fire lasted two turns; the seed and the roots stay.
    game.turn(&["wait", "wait", "wait"]);
    assert_eq!(game.foe(0).hp, 150);
    assert_eq!(game.ally(0).hp, 325);
}

#[test]
fn an_effect_can_change_speed_and_who_moves_first() {
    let swamp = format!(
        "{{id='swamp', name='Swamp', {HEAD}, priority=1,
           script=function(ctx)
             ctx:start_effect(ctx.foe_side, 'swamp', {{turns=2, rules={{{{kind=ctx.Rule.StatMultiplier, stat=ctx.Stat.Speed, factor=0.25}}}}}})
             ctx:message('speed ' .. ctx.target.speed)
           end}}"
    );
    let shout = format!(
        "{{id='shout', name='Shout', {HEAD}, script=function(ctx) ctx:message(ctx.user.name .. ' shouts') end}}"
    );
    let moves = [swamp, shout];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut foe = mon("Foe", &["shout"]);
    foe.speed = 300;
    let mut game = Game::new(&moves, vec![mon("Ally", &["swamp", "shout"])], vec![foe]);
    let order = |lines: &[String]| -> Vec<String> {
        lines
            .iter()
            .filter(|line| line.ends_with("shouts"))
            .cloned()
            .collect()
    };
    assert_eq!(
        order(&game.turn(&["shout", "shout"])),
        ["Foe shouts", "Ally shouts"]
    );
    assert!(said(&game.turn(&["swamp", "shout"]), "speed 75"));
    assert_eq!(
        order(&game.turn(&["shout", "shout"])),
        ["Ally shouts", "Foe shouts"]
    );
    assert_eq!(
        order(&game.turn(&["shout", "shout"])),
        ["Foe shouts", "Ally shouts"]
    );
}

#[test]
fn an_effect_can_block_status_from_others_but_not_from_oneself() {
    let guard = format!(
        "{{id='guard', name='Guard', {HEAD}, target=Target.User, priority=1,
           script=function(ctx)
             ctx:start_effect(ctx.user.team, 'safeguard', {{turns=5, rules={{{{kind=ctx.Rule.BlockStatus, confusion=true}}}}}})
           end}}"
    );
    let spore = format!(
        "{{id='spore', name='Spore', {HEAD},
           script=function(ctx)
             ctx:message('slept ' .. tostring(ctx:apply_status(ctx.target, ctx.Status.Asleep)))
             ctx:message('dizzy ' .. tostring(ctx:confuse(ctx.target)))
           end}}"
    );
    let powder = format!(
        "{{id='powder', name='Powder', {HEAD}, effects={{{{kind=Effect.Status, status=Status.Poisoned}}, {{kind=Effect.Confuse}}}}}}"
    );
    let rest = format!(
        "{{id='rest', name='Rest', {HEAD}, target=Target.User,
           script=function(ctx) ctx:message('rested ' .. tostring(ctx:apply_status(ctx.user, ctx.Status.RestSleep, {{replace=true}}))) ctx:confuse_self() end}}"
    );
    let moves = [guard, spore, powder, rest];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut game = Game::new(
        &moves,
        vec![mon("Ally", &["spore", "powder"])],
        vec![mon("Foe", &["guard", "rest"])],
    );
    let lines = game.turn(&["spore", "guard"]);
    assert!(said(&lines, "slept false"));
    assert!(said(&lines, "dizzy false"));
    game.turn(&["powder", "guard"]);
    assert_eq!(game.foe(0).status, None);
    assert!(game.foe(0).confused_turns.is_none());
    let lines = game.turn(&["spore", "rest"]);
    assert!(said(&lines, "rested true"));
    assert!(game.foe(0).confused_turns.is_some());
}

#[test]
fn only_sleep_can_be_blocked_as_in_an_uproar() {
    let script = test_move(
        "",
        "ctx:start_effect(ctx.field, 'uproar', {turns=3, rules={{kind=ctx.Rule.BlockStatus, statuses={ctx.Status.Asleep}}}})
         for _, pokemon in ipairs(ctx.everyone) do ctx:cure_status(pokemon, ctx.Status.Asleep) end
         ctx:message('sleep ' .. tostring(ctx:apply_status(ctx.target, ctx.Status.Asleep)))
         ctx:message('burn ' .. tostring(ctx:apply_status(ctx.target, ctx.Status.Burned)))",
    );
    let mut sleeper = mon("Sleeper", &["wait"]);
    sleeper.status = Some(Status::Asleep);
    sleeper.status_turns = 3;
    let mut game = Game::new(
        &[&script],
        vec![mon("Ally", &["test"]), sleeper],
        vec![mon("Foe", &["wait"])],
    );
    let lines = game.turn(&["test", "wait", "wait"]);
    assert!(said(&lines, "sleep false"));
    assert!(said(&lines, "burn true"));
    assert_eq!(game.ally(1).status, None);
}

#[test]
fn mist_keeps_stats_from_being_lowered_by_others() {
    let script = test_move(
        "",
        "ctx:start_effect(ctx.foe_side, 'mist', {turns=5, rules={{kind=ctx.Rule.BlockStatDrops}}})
         ctx:message('drop ' .. ctx:change_stat(ctx.target, ctx.Stat.Attack, -1))
         ctx:message('rise ' .. ctx:change_stat(ctx.target, ctx.Stat.Attack, 1))",
    );
    let growl = format!(
        "{{id='growl', name='Growl', {HEAD}, effects={{{{kind=Effect.Stats, stages={{[Stat.Defense]=-1}}}}}}}}"
    );
    let mut game = Game::new(
        &[&script, &growl],
        vec![mon("Ally", &["test", "growl"])],
        vec![mon("Foe", &["wait"])],
    );
    let lines = game.turn(&["test", "wait"]);
    assert!(said(&lines, "drop 0"));
    assert!(said(&lines, "rise 1"));
    assert!(said(
        &game.turn(&["growl", "wait"]),
        "Foe's stats were not lowered!"
    ));
    assert_eq!(game.foe(0).stage(Stat::Defense), 0);
}

#[test]
fn a_rainbow_doubles_the_chances_of_extra_effects() {
    let run = |rainbow: bool| {
        let script = test_move(
            "",
            &format!(
                "if {rainbow} then ctx:start_effect(ctx.user.team, 'rainbow', {{turns=4, rules={{{{kind=ctx.Rule.EffectChance, factor=2}}}}}}) end
                 ctx:message('burn ' .. tostring(ctx:apply_status(ctx.target, ctx.Status.Burned, {{chance=0.3}})))
                 ctx:message('drop ' .. ctx:change_stat(ctx.target, ctx.Stat.Speed, -1, {{chance=0.3}}))"
            ),
        );
        let mut game = Game::duel(&[&script], &["test"]);
        game.turn(&["test", "wait"])
    };
    let without = run(false);
    assert!(said(&without, "burn false") && said(&without, "drop 0"));
    let with = run(true);
    assert!(said(&with, "burn true") && said(&with, "drop -1"));
}

#[test]
fn the_first_flinch_call_follows_the_rainbow_too() {
    let run = |rainbow: bool| {
        let script = test_move(
            "priority=1,",
            &format!(
                "if {rainbow} then ctx:start_effect(ctx.user.team, 'rainbow', {{turns=4, rules={{{{kind=ctx.Rule.EffectChance, factor=2}}}}}}) end
                 ctx:flinch_target(0.3)"
            ),
        );
        let mut game = Game::duel(&[&script], &["test"]);
        game.turn(&["test", "wait"])
    };
    assert!(!said(&run(false), "Foe flinched!"));
    assert!(said(&run(true), "Foe flinched!"));
}

#[test]
fn an_effect_can_change_the_type_of_moves() {
    let run = |deluge: bool| {
        let script = test_move(
            "",
            &format!(
                "if {deluge} then ctx:start_effect(ctx.field, 'ion_deluge', {{turns=1, rules={{{{kind=ctx.Rule.MoveType, from=ctx.Type.Normal, to=ctx.Type.Electric}}}}}}) end
                 ctx:damage(40)"
            ),
        );
        let mut foe = mon("Foe", &["wait"]);
        foe.types = vec![PokemonType::Ground];
        let mut game = Game::new(&[&script], vec![mon("Ally", &["test"])], vec![foe]);
        game.turn(&["test", "wait"]);
        game.dealt()
    };
    assert!(run(false) > 0);
    assert_eq!(
        run(true),
        0,
        "an Electric move does nothing to a Ground type"
    );
}

#[test]
fn grounding_and_losing_a_type() {
    let quake = |setup: &str| {
        let script = format!(
            "{{id='test', name='Test', type=Type.Ground, category=Category.Physical, pp=40,
               script=function(ctx) {setup} local hit = ctx:damage(40) ctx:message('hit ' .. tostring(hit.hit) .. ' types ' .. #ctx.target.types) end}}"
        );
        let mut foe = mon("Bird", &["wait"]);
        foe.types = vec![PokemonType::Normal, PokemonType::Flying];
        let mut game = Game::new(&[&script], vec![mon("Ally", &["test"])], vec![foe]);
        let lines = game.turn(&["test", "wait"]);
        (game.dealt() > 0, lines)
    };
    let (hit, lines) = quake("");
    assert!(!hit && said(&lines, "hit false types 2"));
    let (hit, lines) =
        quake("ctx:start_effect(ctx.target, 'smacked', {rules={{kind=ctx.Rule.Grounded}}})");
    assert!(
        hit && said(&lines, "hit true types 2"),
        "grounded keeps its types: {lines:?}"
    );
    let (hit, lines) = quake(
        "ctx:start_effect(ctx.target, 'roost', {turns=1, rules={{kind=ctx.Rule.WithoutType, type=ctx.Type.Flying}}})",
    );
    assert!(hit && said(&lines, "hit true types 1"));
}

#[test]
fn endure_leaves_one_hp_against_moves() {
    let brace = format!(
        "{{id='brace', name='Brace', {HEAD}, target=Target.User, priority=4,
           script=function(ctx) ctx:start_effect(ctx.user, 'endure', {{turns=1, rules={{{{kind=ctx.Rule.Endure}}}}}}) end}}"
    );
    let ohko = test_move("", "ctx:direct_damage(ctx.target, ctx.target.max_hp)");
    let mut game = Game::new(
        &[&brace, &ohko],
        vec![mon("Ally", &["test"])],
        vec![mon("Foe", &["brace", "wait"])],
    );
    let lines = game.turn(&["test", "brace"]);
    assert!(said(&lines, "Foe endured the hit!"));
    assert_eq!(game.foe(0).hp, 1);
    game.turn(&["test", "wait"]);
    assert_eq!(game.foe(0).hp, 0);
}

#[test]
fn lock_on_makes_the_next_moves_sure_to_hit_even_a_hidden_target() {
    let aim = format!(
        "{{id='aim', name='Aim', {HEAD},
           script=function(ctx) ctx:start_effect(ctx.user, 'lock_on', {{turns=2, rules={{{{kind=ctx.Rule.AlwaysHit, against=ctx.target}}}}}}) end}}"
    );
    let wild = format!(
        "{{id='wild', name='Wild', {HEAD}, accuracy=0.3, effects={{{{kind=Effect.Damage, power=40}}}}}}"
    );
    let moves = [aim, wild, fly()];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut foe = mon("Foe", &["wait", "fly"]);
    foe.speed = 200;
    let mut game = Game::new(&moves, vec![mon("Ally", &["aim", "wild"])], vec![foe]);
    assert!(said(&game.turn(&["wild", "wait"]), "Ally's attack missed!"));
    game.turn(&["aim", "wait"]);
    let lines = game.turn(&["wild", "fly"]);
    assert!(
        !said(&lines, "missed") && !said(&lines, "avoided"),
        "{lines:?}"
    );
    assert!(game.dealt() > 0);
    // The aim is gone after the turn that followed it.
    assert!(game.ally(0).conditions.is_empty());
}

#[test]
fn starting_one_terrain_ends_another() {
    let script = test_move(
        "",
        "if not ctx.field.effects.psychic_terrain then
           ctx:start_effect(ctx.field, 'grassy_terrain', {turns=5, group='terrain'})
           ctx:start_effect(ctx.field, 'psychic_terrain', {turns=5, group='terrain'})
           ctx:message('grassy ' .. tostring(ctx.field.effects.grassy_terrain) .. ' psychic ' .. tostring(ctx.field.effects.psychic_terrain))
         else
           ctx:message('removed ' .. tostring(ctx:end_group(ctx.field, 'terrain')))
         end",
    );
    let mut game = Game::duel(&[&script], &["test"]);
    assert!(said(&game.turn(&["test", "wait"]), "grassy nil psychic 5"));
    assert_eq!(game.battle.field_conditions().len(), 1);
    assert!(said(&game.turn(&["test", "wait"]), "removed true"));
    assert!(game.battle.field_conditions().is_empty());
}

// ---------------------------------------------------------------------------
// What happened this turn, and before

#[test]
fn a_script_knows_who_has_acted_and_what_was_chosen() {
    let script = test_move(
        "",
        "local chosen = ctx.target.selected_move
         ctx:message('acted ' .. tostring(ctx.target.acted) .. ' chose ' .. chosen.id .. ' damaging ' .. tostring(chosen.damaging))",
    );
    let jab =
        format!("{{id='jab', name='Jab', {HEAD}, effects={{{{kind=Effect.Damage, power=10}}}}}}");
    let run = |speed: u16, foe_move: &str| {
        let mut ally = mon("Ally", &["test"]);
        ally.speed = speed;
        let mut game = Game::new(
            &[&script, &jab],
            vec![ally],
            vec![mon("Foe", &["wait", "jab"])],
        );
        game.turn(&["test", foe_move])
    };
    assert!(said(
        &run(200, "jab"),
        "acted false chose jab damaging true"
    ));
    assert!(said(
        &run(50, "wait"),
        "acted true chose wait damaging false"
    ));
}

#[test]
fn a_script_knows_what_hit_its_user_this_turn() {
    let revenge = test_move(
        "priority=-4,",
        "local attacker = ctx.user.last_attacker
         ctx:message('taken ' .. ctx.user.damage_taken .. ' by target ' .. tostring(attacker == ctx.target) .. ' hurt ' .. tostring(ctx.target.hurt_this_turn))
         if ctx.user.last_hit then ctx:message(ctx.user.last_hit.move .. ' ' .. tostring(ctx.user.last_hit.category == ctx.Category.Physical)) end",
    );
    let jab =
        format!("{{id='jab', name='Jab', {HEAD}, effects={{{{kind=Effect.Damage, power=10}}}}}}");
    let mut game = Game::new(
        &[&revenge, &jab],
        vec![mon("Ally", &["test"])],
        vec![mon("Foe", &["wait", "jab"])],
    );
    let quiet = game.turn(&["test", "wait"]);
    assert!(said(&quiet, "taken 0 by target false hurt false"));
    let lost = 400 - game.ally(0).hp;
    assert_eq!(lost, 0);
    let lines = game.turn(&["test", "jab"]);
    let lost = 400 - game.ally(0).hp;
    assert!(
        said(&lines, &format!("taken {lost} by target true hurt false")),
        "{lines:?}"
    );
    assert!(said(&lines, "jab true"));
}

#[test]
fn a_script_knows_how_long_its_user_has_been_out_and_what_it_has_used() {
    let fake_out = format!(
        "{{id='fake_out', name='Fake Out', {HEAD}, priority=3,
           script=function(ctx)
             if ctx.user.turns_on_field > 1 then return ctx:fail() end
             ctx:damage(40) ctx:flinch(ctx.target)
           end}}"
    );
    let last_resort = format!(
        "{{id='last_resort', name='Last Resort', {HEAD},
           script=function(ctx)
             for _, id in ipairs(ctx.user.moves) do
               if id ~= ctx.move.id and not ctx.user.used_moves[id] then return ctx:fail() end
             end
             if #ctx.user.moves == 1 then return ctx:fail() end
             ctx:damage(140)
           end}}"
    );
    let moves = [fake_out, last_resort];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut game = Game::new(
        &moves,
        vec![mon("Ally", &["fake_out", "last_resort"])],
        vec![mon("Foe", &["wait"])],
    );
    assert!(said(&game.turn(&["last_resort", "wait"]), "But it failed!"));
    assert_eq!(game.dealt(), 0);
    let lines = game.turn(&["fake_out", "wait"]);
    assert!(
        said(&lines, "But it failed!"),
        "only on the first turn out: {lines:?}"
    );
    game.turn(&["last_resort", "wait"]);
    assert!(game.dealt() > 0);

    let mut game = Game::new(
        &moves,
        vec![mon("Ally", &["fake_out", "last_resort"])],
        vec![mon("Foe", &["wait"])],
    );
    let lines = game.turn(&["fake_out", "wait"]);
    assert!(said(&lines, "Foe flinched!"));
}

#[test]
fn a_script_knows_whether_the_last_move_failed() {
    let tantrum = test_move(
        "",
        "local power = 75
         if ctx.user.last_move_failed then power = power * 2 end
         ctx:message('power ' .. power)",
    );
    let flop =
        format!("{{id='flop', name='Flop', {HEAD}, script=function(ctx) return ctx:fail() end}}");
    let wild = format!(
        "{{id='wild', name='Wild', {HEAD}, accuracy=0.3, effects={{{{kind=Effect.Damage, power=40}}}}}}"
    );
    let spook =
        "{id='spook', name='Spook', type=Type.Ghost, category=Category.Physical, pp=40, effects={{kind=Effect.Damage, power=40}}}"
            .to_string();
    let moves = [tantrum, flop, wild, spook];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut game = Game::new(
        &moves,
        vec![mon("Ally", &["test", "flop", "wild", "spook"])],
        vec![mon("Foe", &["wait"])],
    );
    assert!(said(&game.turn(&["test", "wait"]), "power 75"));
    for failing in ["flop", "wild", "spook"] {
        game.turn(&[failing, "wait"]);
        assert!(game.ally(0).last_move_failed, "{failing}");
        assert!(
            said(&game.turn(&["test", "wait"]), "power 150"),
            "{failing}"
        );
        assert!(said(&game.turn(&["test", "wait"]), "power 75"));
    }
}

#[test]
fn a_script_knows_when_an_ally_fainted_last_turn() {
    let retaliate = test_move(
        "",
        "ctx:message('fallen ' .. tostring(ctx.user.team.fainted_last_turn))",
    );
    let boom = format!(
        "{{id='boom', name='Boom', {HEAD}, target=Target.User, script=function(ctx) ctx:faint_user() end}}"
    );
    let mut game = Game::new(
        &[&retaliate, &boom],
        vec![mon("Ally", &["test"]), mon("Partner", &["boom"])],
        vec![mon("Foe", &["wait"])],
    );
    assert!(said(&game.turn(&["test", "boom", "wait"]), "fallen false"));
    assert!(said(&game.turn(&["test", "wait"]), "fallen true"));
    assert!(said(&game.turn(&["test", "wait"]), "fallen false"));
}

// ---------------------------------------------------------------------------
// Hooks

#[test]
fn a_move_can_get_ready_at_the_start_of_the_turn() {
    let focus = format!(
        "{{id='focus', name='Focus', {HEAD}, priority=-3,
           on_turn_start=function(ctx) ctx:message(ctx.user.name .. ' is tightening its focus!') end,
           script=function(ctx)
             if ctx.user.damage_taken > 0 then ctx:message(ctx.user.name .. ' lost its focus!') return ctx:fail() end
             ctx:damage(150)
           end}}"
    );
    let jab =
        format!("{{id='jab', name='Jab', {HEAD}, effects={{{{kind=Effect.Damage, power=10}}}}}}");
    let moves = [focus, jab];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut game = Game::new(
        &moves,
        vec![mon("Ally", &["focus"])],
        vec![mon("Foe", &["wait", "jab"])],
    );
    let lines = game.turn(&["focus", "jab"]);
    assert_eq!(lines[0], "Ally is tightening its focus!");
    assert!(said(&lines, "Ally lost its focus!"));
    assert_eq!(game.dealt(), 0);
    game.turn(&["focus", "wait"]);
    assert!(game.dealt() > 0);
}

#[test]
fn a_hit_reaction_can_act_on_the_attacker_and_knows_the_hit() {
    let beak = format!(
        "{{id='beak', name='Beak', {HEAD}, priority=-3,
           on_turn_start=function(ctx) ctx:watch_hits_until_next_action() ctx:message(ctx.user.name .. ' heated up its beak!') end,
           on_hit=function(ctx)
             if ctx.hit.contact then ctx:apply_status(ctx.target, ctx.Status.Burned) end
           end,
           script=function(ctx) ctx:damage(100) end}}"
    );
    let jab = format!(
        "{{id='jab', name='Jab', {HEAD}, flags={{'contact'}}, effects={{{{kind=Effect.Damage, power=10}}}}}}"
    );
    let beam =
        format!("{{id='beam', name='Beam', {HEAD}, effects={{{{kind=Effect.Damage, power=10}}}}}}");
    let moves = [beak, jab, beam];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut game = Game::new(
        &moves,
        vec![mon("Ally", &["beak"])],
        vec![mon("Foe", &["jab", "beam"])],
    );
    game.turn(&["beak", "beam"]);
    assert_eq!(game.foe(0).status, None);
    let lines = game.turn(&["beak", "jab"]);
    assert!(said(&lines, "Ally heated up its beak!"));
    assert!(said(&lines, "Foe was burned!"));
}

#[test]
fn hooks_that_react_cannot_start_attacks() {
    let bad = format!(
        "{{id='bad', name='Bad', {HEAD},
           on_turn_start=function(ctx) ctx:damage(10) end,
           script=function(ctx) end}}"
    );
    let mut game = Game::duel(&[&bad], &["bad"]);
    let result = (|| {
        loop {
            let result = game.battle.advance()?;
            let AdvanceStatus::Awaiting(prompt) = result.status else {
                return Ok(());
            };
            game.battle
                .set_response(ActionSelection {
                    prompt_id: prompt.id,
                    choice_id: prompt.choices[0].id(),
                })
                .unwrap();
        }
    })();
    let error: wilds_battle_engine::BattleError = result.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("turn start hook for bad yielded an unsupported operation")
    );
}

#[test]
fn round_lets_its_other_users_act_right_away() {
    let round = format!(
        "{{id='round', name='Round', {HEAD},
           script=function(ctx)
             local power = 60
             if ctx.field.marks.round then power = power * 2 end
             ctx:message(ctx.user.name .. ' sings with power ' .. power)
             ctx:mark(ctx.field, 'round', 1, 1)
             ctx:hurry()
           end}}"
    );
    let shout = format!(
        "{{id='shout', name='Shout', {HEAD}, script=function(ctx) ctx:message(ctx.user.name .. ' shouts') end}}"
    );
    let moves = [round, shout];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut fast = mon("Fast", &["round"]);
    fast.speed = 300;
    let mut middle = mon("Middle", &["shout"]);
    middle.speed = 200;
    let slow = mon("Slow", &["round"]);
    let mut game = Game::new(&moves, vec![fast, slow], vec![middle]);
    let lines = game.turn(&["round", "round", "shout"]);
    let spoken: Vec<&String> = lines
        .iter()
        .filter(|line| line.contains("sings") || line.contains("shouts"))
        .collect();
    assert_eq!(
        spoken,
        [
            "Fast sings with power 60",
            "Slow sings with power 120",
            "Middle shouts"
        ]
    );
}

#[test]
fn a_frozen_pokemon_thaws_to_use_a_move_that_says_so() {
    let wheel = format!(
        "{{id='wheel', name='Wheel', {HEAD}, usable_while_frozen=true, effects={{{{kind=Effect.Damage, power=60}}}}}}"
    );
    let jab =
        format!("{{id='jab', name='Jab', {HEAD}, effects={{{{kind=Effect.Damage, power=10}}}}}}");
    let moves = [wheel, jab];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let frozen = |id: &str| {
        let mut ally = mon("Ally", &[id]);
        ally.status = Some(Status::Frozen);
        let mut game = Game::new(&moves, vec![ally], vec![mon("Foe", &["wait"])]);
        let lines = game.turn(&[id, "wait"]);
        (game.dealt() > 0, said(&lines, "Ally thawed out!"))
    };
    assert_eq!(frozen("jab"), (false, false));
    assert_eq!(frozen("wheel"), (true, true));
}

// ---------------------------------------------------------------------------
// More than one target

#[test]
fn moves_can_be_aimed_at_allies_and_at_whole_sides() {
    let cheer = format!(
        "{{id='cheer', name='Cheer', {HEAD}, target=Target.Ally,
           script=function(ctx) ctx:change_stat(ctx.target, ctx.Stat.SpDefense, 1) end}}"
    );
    let mend = format!(
        "{{id='mend', name='Mend', {HEAD}, target=Target.UserOrAlly,
           script=function(ctx) ctx:heal(ctx.target, {{fraction=1/2}}) end}}"
    );
    let wave = format!(
        "{{id='wave', name='Wave', {HEAD}, target=Target.AllOpponents,
           script=function(ctx)
             local hit = ctx:damage(40)
             ctx:message('struck ' .. hit.hits .. ' of ' .. #ctx.targets)
           end}}"
    );
    let rally = format!(
        "{{id='rally', name='Rally', {HEAD}, target=Target.UserAndAllies,
           script=function(ctx) for _, friend in ipairs(ctx.targets) do ctx:change_stat(friend, ctx.Stat.Attack, 1) end end}}"
    );
    let moves = [cheer, mend, wave, rally];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let mut hurt = mon("Partner", &["wait"]);
    hurt.hp = 100;
    let mut game = Game::new(
        &moves,
        vec![mon("Ally", &["cheer", "mend", "wave", "rally"]), hurt],
        vec![mon("Foe", &["wait"]), mon("Other", &["wait"])],
    );
    game.turn(&["cheer", "wait", "wait", "wait"]);
    assert_eq!(game.ally(1).stage(Stat::SpDefense), 1);
    assert_eq!(game.ally(0).stage(Stat::SpDefense), 0);
    game.turn(&["mend>2", "wait", "wait", "wait"]);
    assert_eq!(game.ally(1).hp, 300);
    let lines = game.turn(&["wave", "wait", "wait", "wait"]);
    assert!(said(&lines, "struck 2 of 2"));
    assert!(game.foe(0).hp < 400 && game.foe(1).hp < 400);
    assert_eq!(game.ally(1).hp, 300);
    game.turn(&["rally", "wait", "wait", "wait"]);
    assert_eq!(
        (
            game.ally(0).stage(Stat::Attack),
            game.ally(1).stage(Stat::Attack)
        ),
        (1, 1)
    );
}

#[test]
fn a_move_can_be_aimed_at_a_foe_or_an_ally() {
    let puff = format!(
        "{{id='puff', name='Puff', {HEAD}, target=Target.AnyOther,
           script=function(ctx)
             if ctx.target.is_ally then ctx:heal(ctx.target, {{fraction=1/2}}) else ctx:damage(90) end
           end}}"
    );
    let mut hurt = mon("Partner", &["wait"]);
    hurt.hp = 100;
    let mut game = Game::new(
        &[&puff],
        vec![mon("Ally", &["puff"]), hurt],
        vec![mon("Foe", &["wait"])],
    );
    // The second Pokémon on offer is the partner, the first is the foe; the user is never offered.
    game.turn(&["puff>2", "wait", "wait"]);
    assert_eq!(game.ally(1).hp, 300);
    assert_eq!(game.dealt(), 0);
    game.turn(&["puff>1", "wait", "wait"]);
    assert!(game.dealt() > 0);
    assert_eq!(game.ally(0).hp, 400);
}

#[test]
fn a_script_can_single_out_any_pokemon() {
    let burst = test_move(
        "",
        "local hit = ctx:damage(70)
         if hit.hit then
           for _, other in ipairs(ctx.target.team.pokemon) do
             if other ~= ctx.target then ctx:hurt(other, {fraction=1/16}) end
           end
         end
         ctx:message('foes ' .. #ctx.foes .. ' allies ' .. #ctx.allies .. ' others ' .. #ctx.others)
         ctx:damage(20, {target=ctx.allies[1]})",
    );
    let mut game = Game::new(
        &[&burst],
        vec![mon("Ally", &["test"]), mon("Partner", &["wait"])],
        vec![mon("Foe", &["wait"]), mon("Other", &["wait"])],
    );
    let lines = game.turn(&["test>1", "wait", "wait", "wait"]);
    assert!(said(&lines, "foes 2 allies 1 others 3"));
    assert!(game.foe(0).hp < 375);
    assert_eq!(game.foe(1).hp, 375);
    assert!(game.ally(1).hp < 400);
}

// ---------------------------------------------------------------------------
// Things the engine only stores

#[test]
fn items_can_be_read_taken_and_given() {
    let thief = test_move(
        "",
        "ctx:damage(40)
         if ctx.target.item and not ctx.user.item then
           local item = ctx:take_item(ctx.target)
           ctx:give_item(ctx.user, item)
           ctx:message(ctx.user.name .. ' stole ' .. item .. ' and holds ' .. tostring(ctx.user.item) .. ', target ' .. tostring(ctx.target.item))
         end",
    );
    let mut foe = mon("Foe", &["wait"]);
    foe.item = Some("oran_berry".into());
    let mut game = Game::new(&[&thief], vec![mon("Ally", &["test"])], vec![foe]);
    let lines = game.turn(&["test", "wait"]);
    assert!(said(
        &lines,
        "Ally stole oran_berry and holds oran_berry, target nil"
    ));
    assert_eq!(game.ally(0).item.as_deref(), Some("oran_berry"));
    assert_eq!(game.foe(0).item, None);
}

#[test]
fn gender_weight_ability_species_and_ivs_are_there_to_read() {
    let script = test_move(
        "",
        "local same = ctx.user.gender == ctx.target.gender or ctx.user.gender == ctx.Gender.Genderless
         ctx:message('same ' .. tostring(same) .. ' weight ' .. ctx.target.weight .. ' ability ' .. tostring(ctx.target.ability) .. ' species ' .. ctx.user.species)
         ctx:set_weight(ctx.user, ctx.user.weight / 2)
         ctx:message('hushed ' .. tostring(ctx:suppress_ability(ctx.target)) .. ' ' .. tostring(ctx.target.ability))
         local ivs = ctx.user.ivs
         ctx:message('ivs ' .. (ivs and ivs.attack + ivs.speed or -1) .. ' happy ' .. tostring(ctx.user.happiness))",
    );
    let mut ally = mon("Ally", &["test"]);
    ally.gender = Gender::Male;
    ally.weight = 60.0;
    ally.species = "chatot".into();
    ally.ivs = Some([1, 2, 3, 4, 5, 6]);
    ally.happiness = Some(255);
    let mut foe = mon("Foe", &["wait"]);
    foe.gender = Gender::Female;
    foe.weight = 12.5;
    foe.ability = Some("levitate".into());
    let mut game = Game::new(&[&script], vec![ally], vec![foe]);
    let lines = game.turn(&["test", "wait"]);
    assert!(said(
        &lines,
        "same false weight 12.5 ability levitate species chatot"
    ));
    assert!(said(&lines, "hushed true nil"));
    assert!(said(&lines, "ivs 8 happy 255"));
    assert_eq!(game.ally(0).weight, 30.0);
    assert!(game.foe(0).ability_suppressed);
}

#[test]
fn types_can_be_changed() {
    let burn_up =
        "{id='burn_up', name='Burn Up', type=Type.Fire, category=Category.Special, pp=5,
           script=function(ctx)
             if not ctx.user:has_type(ctx.Type.Fire) then return ctx:fail() end
             ctx:damage(130)
             local kept = {}
             for _, kind in ipairs(ctx.user.types) do if kind ~= ctx.Type.Fire then kept[#kept + 1] = kind end end
             if #kept == 0 then kept = {ctx.Type.None} end
             ctx:set_types(ctx.user, kept)
           end}"
            .to_string();
    let mut ally = mon("Ally", &["burn_up"]);
    ally.types = vec![PokemonType::Fire, PokemonType::Flying];
    let mut game = Game::new(&[&burn_up], vec![ally], vec![mon("Foe", &["wait"])]);
    game.turn(&["burn_up", "wait"]);
    assert!(game.dealt() > 0);
    assert_eq!(game.ally(0).types, [PokemonType::Flying]);
    assert!(said(&game.turn(&["burn_up", "wait"]), "But it failed!"));
}

#[test]
fn money_and_the_place_of_the_battle() {
    let pay_day = test_move(
        "",
        "ctx:damage(40)
         ctx:add_payout(5 * ctx.user.level)
         ctx:message('place ' .. tostring(ctx.environment) .. ' turn ' .. ctx.battle_turn)",
    );
    let mut game = Game::duel(&[&pay_day], &["test"]);
    game.battle.set_environment(Some("cave".into()));
    assert!(said(&game.turn(&["test", "wait"]), "place cave turn 1"));
    game.turn(&["test", "wait"]);
    assert_eq!(game.battle.payout(Side::Allies), 1000);
    assert_eq!(game.battle.payout(Side::Foes), 0);
}

#[test]
fn binding_and_freeing() {
    let script = test_move(
        "",
        "if not ctx.target.bound then
           ctx:message('bound ' .. tostring(ctx:bind(ctx.target, {min_turns=2, max_turns=2})) .. ' trapped ' .. tostring(ctx.target.trapped))
         end",
    );
    let spin = format!(
        "{{id='spin', name='Spin', {HEAD},
           script=function(ctx) ctx:damage(20) ctx:message('freed ' .. tostring(ctx:free(ctx.user))) end}}"
    );
    let mut game = Game::new(
        &[&script, &spin],
        vec![mon("Ally", &["test"])],
        vec![mon("Foe", &["wait", "spin"])],
    );
    let lines = game.turn(&["test", "wait"]);
    assert!(said(&lines, "Foe was squeezed by Ally!"));
    assert!(said(&lines, "bound true trapped true"));
    assert!(game.battle.trapped(wilds_battle_engine::ParticipantId {
        side: Side::Foes,
        index: 0
    }));
    assert!(said(&game.turn(&["test", "spin"]), "freed true"));
    assert!(game.foe(0).bound.is_none());
}

// ---------------------------------------------------------------------------
// Shorthands

#[test]
fn effects_on_a_target_can_wait_for_the_move_to_reach_it() {
    let guard = format!(
        "{{id='guard', name='Guard', {HEAD}, target=Target.User, priority=4,
           script=function(ctx) ctx:protect() end}}"
    );
    // A status move: the first thing it does to the target makes the usual checks, once.
    let spore = format!(
        "{{id='spore', name='Spore', {HEAD}, accuracy=0.4,
           script=function(ctx)
             if ctx:reached() then ctx:apply_status(ctx.target, ctx.Status.Asleep) end
             if ctx:reached() then ctx:change_stat(ctx.target, ctx.Stat.Speed, -1) end
             ctx:message('user ' .. tostring(ctx:reached(ctx.user)))
           end}}"
    );
    // An attack: what follows depends on the hit that was already made.
    let bite = format!(
        "{{id='bite', name='Bite', {HEAD},
           script=function(ctx)
             ctx:damage(40)
             if ctx:reached() then ctx:change_stat(ctx.target, ctx.Stat.Defense, -1) end
           end}}"
    );
    let moves = [guard, spore, bite];
    let moves: Vec<&str> = moves.iter().map(String::as_str).collect();
    let fresh = |dice: f32| {
        Game::with_dice(
            dice,
            &moves,
            vec![mon("Ally", &["spore", "bite"])],
            vec![mon("Foe", &["wait", "guard"])],
        )
    };
    let mut game = fresh(0.5);
    let lines = game.turn(&["spore", "wait"]);
    assert_eq!(
        lines.iter().filter(|line| line.contains("missed")).count(),
        1,
        "one check for the whole move: {lines:?}"
    );
    assert!(said(&lines, "user true"));
    assert_eq!(game.foe(0).status, None);
    assert_eq!(game.foe(0).stage(Stat::Speed), 0);

    let mut game = fresh(0.3);
    game.turn(&["spore", "wait"]);
    assert_eq!(game.foe(0).status, Some(Status::Asleep));
    assert_eq!(game.foe(0).stage(Stat::Speed), -1);

    let mut game = fresh(0.5);
    game.turn(&["bite", "guard"]);
    assert_eq!(
        game.foe(0).stage(Stat::Defense),
        0,
        "blocked: nothing follows"
    );
    game.turn(&["bite", "wait"]);
    assert_eq!(game.foe(0).stage(Stat::Defense), -1);
}

#[test]
fn shorthands_for_boosts_items_and_types() {
    let script = test_move(
        "",
        "ctx:change_stat(ctx.target, ctx.Stat.Attack, 2)
         ctx:change_stat(ctx.target, ctx.Stat.Speed, -1)
         ctx:message('raised ' .. ctx.target:raised_stages())
         ctx:steal_boosts(ctx.target, ctx.user)
         ctx:message('now ' .. ctx.target:stage(ctx.Stat.Attack) .. ' ' .. ctx.user:stage(ctx.Stat.Attack) .. ' ' .. ctx.target:stage(ctx.Stat.Speed))
         ctx:message('stole ' .. tostring(ctx:steal_item(ctx.target, ctx.user)) .. ' again ' .. tostring(ctx:steal_item(ctx.target, ctx.user)))
         ctx:lose_type(ctx.user, ctx.Type.Fire)
         ctx:message('types ' .. #ctx.user.types .. ' fire ' .. tostring(ctx.user:has_type(ctx.Type.Fire)))
         ctx:message('shared ' .. tostring(ctx:share_type(ctx.user, ctx.target)) .. ' opposite ' .. tostring(ctx:opposite_genders(ctx.user, ctx.target)))
         ctx:message('ninth ' .. tostring(ctx:type_number(9) == ctx.Type.Water) .. ' all used ' .. tostring(ctx.user:used_all_other_moves(ctx.move.id)))",
    );
    let mut ally = mon("Ally", &["test", "wait"]);
    ally.types = vec![PokemonType::Fire, PokemonType::Flying];
    ally.gender = Gender::Male;
    let mut foe = mon("Foe", &["wait"]);
    foe.types = vec![PokemonType::Flying];
    foe.gender = Gender::Female;
    foe.item = Some("leftovers".into());
    let mut game = Game::new(&[&script], vec![ally], vec![foe]);
    let lines = game.turn(&["test", "wait"]);
    for expected in [
        "raised 2",
        "now 0 2 -1",
        "stole leftovers again nil",
        "types 1 fire false",
        "shared true opposite true",
        "ninth true all used false",
    ] {
        assert!(said(&lines, expected), "{expected} in {lines:?}");
    }
}

#[test]
fn an_effect_chance_follows_the_rainbow() {
    let script = test_move(
        "",
        "ctx:message('plain ' .. tostring(ctx:chance(0.3)) .. ' effect ' .. tostring(ctx:effect_chance(0.3)))
         ctx:start_effect(ctx.user.team, 'rainbow', {turns=4, rules={{kind=ctx.Rule.EffectChance, factor=2}}})
         ctx:message('then ' .. tostring(ctx:chance(0.3)) .. ' effect ' .. tostring(ctx:effect_chance(0.3)))",
    );
    let mut game = Game::duel(&[&script], &["test"]);
    let lines = game.turn(&["test", "wait"]);
    assert!(said(&lines, "plain false effect false"));
    assert!(said(&lines, "then false effect true"));
}

// ---------------------------------------------------------------------------
// Guard rails

#[test]
fn mistakes_in_a_script_are_reported() {
    let failing = |body: &str| -> String {
        let script = test_move("", body);
        let mut game = Game::duel(&[&script], &["test"]);
        loop {
            match game.battle.advance() {
                Err(error) => return error.to_string(),
                Ok(result) => {
                    let AdvanceStatus::Awaiting(prompt) = result.status else {
                        panic!("no error")
                    };
                    game.battle
                        .set_response(ActionSelection {
                            prompt_id: prompt.id,
                            choice_id: prompt.choices[0].id(),
                        })
                        .unwrap();
                }
            }
        }
    };
    assert!(
        failing("ctx:apply_status(ctx.target, ctx.Stat.Attack)")
            .contains("requires a Status value")
    );
    assert!(failing("ctx:apply_status('foe', ctx.Status.Burned)").contains("needs a Pokémon"));
    assert!(failing("ctx:apply_status(ctx.field, ctx.Status.Burned)").contains("needs a Pokémon"));
    assert!(failing("ctx:damage(40, {tipe=ctx.Type.Fire})").contains("unknown damage option tipe"));
    assert!(failing("ctx:heal(ctx.user, {fraction=2})").contains("must be between 0 and 1"));
    assert!(
        failing(
            "ctx:start_effect(ctx.field, 'x', {rules={{kind=ctx.Rule.StatMultiplier, factor=2}}})"
        )
        .contains("needs stat")
    );
    assert!(failing("ctx:mark(ctx.user, '')").contains("1 to 40 characters"));
    assert!(failing("while true do ctx:message('x') end").contains("exceeded 256 operations"));
}
