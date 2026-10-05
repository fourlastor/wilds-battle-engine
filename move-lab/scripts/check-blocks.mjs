// Checks that the blocks do what their words and the "All blocks" page say:
// every block is explained and writes at least one line, a block that says "the user" acts on the
// user in the real engine whoever the move is aimed at, and a multi-turn move aims its later turns
// the way its menu says.
// Run with: npm run test:reference
import assert from 'node:assert/strict';
import { Blockly, engine, originals } from './headless.mjs';

const { HAT_TYPES, optionSpecs } = await import('../src/blocks/definitions.ts');
const { formsOf, generate } = await import('../src/blocks/generate.ts');
const { newMoveSheet, workspaceState } = await import('../src/blocks/library.ts');
const { OPTION_FIELDS, REFERENCE } = await import('../src/blocks/reference.ts');
const { CATEGORIES } = await import('../src/blocks/toolbox.ts');

const workspace = new Blockly.Workspace();
let checks = 0;
const check = (name, run) => {
  run();
  checks += 1;
  console.log(`ok   ${name}`);
};

// ---------------------------------------------------------------------------
// The reference

check('every block of the palette is on the page, explained, with the line it writes', () => {
  const listed = new Set();
  for (const entry of REFERENCE.flatMap((section) => section.entries)) {
    const block = Blockly.serialization.blocks.append(entry.state, workspace).getDescendants(false).find((inside) => inside.type === entry.type);
    assert.ok(block, `${entry.type}: its example does not contain the block`);
    listed.add(entry.type);
    assert.ok(block.getTooltip().trim().length > 20, `${entry.type} has no explanation`);
    const forms = formsOf(block);
    assert.ok(forms.plain !== null || forms.script !== null, `${entry.type} writes nothing`);
    if (entry.variant) continue;
    for (const spec of optionSpecs(entry.type)) {
      const id = (spec.repeat ?? 1) > 1 ? `${spec.key}#1` : spec.key;
      const fields = { ...entry.state.fields, ...OPTION_FIELDS[`${entry.type}/${spec.key}`] };
      const withOption = formsOf(Blockly.serialization.blocks.append({ ...entry.state, fields, extraState: { options: [id] } }, workspace));
      const changes = (withOption.plain !== null && withOption.plain !== forms.plain) || (withOption.script !== null && withOption.script !== forms.script);
      assert.ok(changes, `${entry.type}: the option “${spec.menu}” changes no line`);
    }
  }
  const palette = [...HAT_TYPES, ...CATEGORIES.flatMap((category) => category.blocks.map((block) => block.type))];
  assert.deepEqual([...listed].sort(), [...new Set(palette)].sort());
});

// ---------------------------------------------------------------------------
// What the blocks do in the engine

const number = (value) => ({ shadow: { type: 'mlab_number', fields: { NUM: value } } });
const heal = () => ({ type: 'mlab_heal', fields: { PERCENT: 50 } });
const damage = () => ({ type: 'mlab_damage', inputs: { POWER: number(40) } });
const protect = () => ({ type: 'mlab_protect' });
const poison = () => ({ type: 'mlab_status', fields: { STATUS: 'Poisoned', CHANCE: 100 } });
const message = () => ({ type: 'mlab_message', fields: { TEXT: 'Hello!' } });
const sunlight = () => ({ type: 'mlab_weather', fields: { WEATHER: 'Sun', TURNS: 5 } });
const onFirstTurn = (then) => ({ type: 'mlab_if', inputs: { COND: { block: { type: 'mlab_first_turn' } }, DO: { block: then } } });
const threeTurns = (policy) => ({ type: 'mlab_force_move', fields: { POLICY: policy }, inputs: { TURNS: number(3) } });

/** The move file for a stack of blocks under "when this move is used on <target>". */
function build(target, stack) {
  workspace.clear();
  Blockly.serialization.workspaces.load(workspaceState(target, stack), workspace);
  return generate(workspace, newMoveSheet('Check', 'check'));
}

const mon = (name, moves, extra = {}) => ({
  name, moves, level: 40, max_hp: 140, hp: 140, attack: 70, defense: 60, sp_attack: 75, sp_defense: 65, speed: 50, types: ['Normal'], ...extra,
});

/** Plays one turn in which the first ally uses the move, and returns the state after it. */
function playTurn(generated, allies, foes) {
  assert.deepEqual(generated.problems, []);
  engine.setMoves({ ...originals, 'check.lua': generated.lua });
  const battle = engine.createBattle({ seed: 3, allies, foes });
  try {
    for (let step = 0; step < 12; step += 1) {
      const result = battle.advance();
      if (result.status.kind === 'end' || result.turn >= 1) break;
      const { choices, prompt_id: promptId } = result.status;
      battle.respond(promptId, (choices.find((choice) => choice.move_id === 'check') ?? choices[0]).id);
    }
    return battle.state();
  } finally {
    battle.dispose();
  }
}

// Everyone but the first ally spends the turn on Sunny Day, which hurts no one.
const bystander = (name) => mon(name, ['sunny_day']);

check('“heal the user” in a move used on the user is a plain move and heals the user', () => {
  const generated = build('User', [heal()]);
  assert.equal(generated.mode, 'effects');
  const state = playTurn(generated, [mon('Ally', ['check'], { hp: 50 })], [bystander('Foe')]);
  assert.equal(state.allies[0].hp, 120);
});

check('“heal the user” in a move used on a foe becomes a script and still heals the user', () => {
  const generated = build('Selected', [heal()]);
  assert.equal(generated.mode, 'script');
  const state = playTurn(generated, [mon('Ally', ['check'], { hp: 50 })], [bystander('Foe')].map((foe) => ({ ...foe, hp: 60 })));
  assert.equal(state.allies[0].hp, 120);
  assert.equal(state.foes[0].hp, 60);
});

check('“deal damage” used on “everyone else” becomes a script and hits every other Pokémon', () => {
  const generated = build('AllOthers', [damage()]);
  assert.equal(generated.mode, 'script');
  const state = playTurn(generated, [mon('Ally', ['check']), bystander('Partner')], [bystander('Foe'), bystander('Other')]);
  assert.equal(state.allies[0].hp, 140);
  for (const other of [state.allies[1], ...state.foes]) assert.ok(other.hp < 140, `${other.name} was not hit`);
});

check('“protect the user” is only accepted in a move used on the user', () => {
  assert.deepEqual(build('User', [protect()]).problems, []);
  const problems = build('Selected', [protect()]).problems;
  assert.equal(problems.length, 1);
  assert.match(problems[0].message, /protects whoever the move is used on/);
});

check('the engine’s ready-made multi-turn blocks still need a plain move, and say why they do not fit', () => {
  const charge = { type: 'mlab_two_turn', fields: { MESSAGE: '{user} is glowing!', POWER: 80 } };
  const problems = build('Selected', [charge, message()]).problems;
  assert.equal(problems.length, 1);
  assert.match(problems[0].message, /only works in plain moves for now\. This move is a script because of “show message”\./);
});

// ---------------------------------------------------------------------------
// Blocks that used to be for plain moves only, and the newer ones

/** Plays turns until `turns` have passed and returns the state and every message shown. */
function playTurns(generated, allies, foes, turns = 1, { extra = {}, picks = () => undefined, setup = {} } = {}) {
  assert.deepEqual(generated.problems, []);
  engine.setMoves({ ...originals, ...extra, 'check.lua': generated.lua });
  const battle = engine.createBattle({ seed: 3, allies, foes, ...setup });
  const messages = [];
  try {
    for (let step = 0; step < 60; step += 1) {
      const result = battle.advance();
      for (const event of result.events) if (event.kind === 'message') messages.push(event.text);
      if (result.status.kind === 'end' || result.turn >= turns) break;
      const { choices, prompt_id: promptId, actor } = result.status;
      const wanted = picks(actor, result.turn + 1) ?? 'check';
      battle.respond(promptId, (choices.find((choice) => choice.move_id === wanted) ?? choices[0]).id);
    }
    return { state: battle.state(), messages };
  } finally {
    battle.dispose();
  }
}

const when = (condition, then, otherwise) => ({
  type: otherwise ? 'mlab_if_else' : 'mlab_if',
  inputs: { COND: { block: condition }, DO: { block: then }, ...(otherwise ? { ELSE: { block: otherwise } } : {}) },
});
const chain = (...blocks) => {
  blocks.forEach((block, index) => {
    if (index + 1 < blocks.length) block.next = { block: blocks[index + 1] };
  });
  return blocks[0];
};
const strike = (power, options = [], fields = {}) => ({
  type: 'mlab_damage',
  fields,
  inputs: { POWER: typeof power === 'number' ? number(power) : { ...number(40), block: power } },
  ...(options.length ? { extraState: { options } } : {}),
});
const lowerDefense = () => ({ type: 'mlab_stats', fields: { WHO: 'target', STAT: 'Defense', STAGES: '-1' } });
const guard = `return { id="guard", name="Guard", type=Type.Normal, category=Category.Status, pp=10, target=Target.User, priority=4,
  script=function(ctx) ctx:protect() end }`;

check('status, stat drops and messages now mix in one move, and wait for the move to get through', () => {
  const generated = build('Selected', [poison(), lowerDefense(), message()]);
  assert.equal(generated.mode, 'script');
  const open = playTurns(generated, [mon('Ally', ['check'])], [bystander('Foe')]);
  assert.equal(open.state.foes[0].status, 'Poisoned');
  assert.deepEqual(open.state.foes[0].stages, [['Defense', -1]]);
  assert.ok(open.messages.includes('Hello!'));

  const blocked = playTurns(generated, [mon('Ally', ['check'])], [mon('Foe', ['guard'])], 1, {
    extra: { 'guard.lua': guard },
    picks: (actor) => (actor.side === 'foes' ? 'guard' : 'check'),
  });
  assert.equal(blocked.state.foes[0].status, null);
  assert.deepEqual(blocked.state.foes[0].stages, []);
  // The move is stopped once, and what does not touch the target still happens.
  assert.equal(blocked.messages.filter((text) => text === 'Foe protected itself!').length, 2);
  assert.ok(blocked.messages.includes('Hello!'));
});

check('an extra effect after a hit follows that hit', () => {
  assert.equal(build('Selected', [strike(40), lowerDefense()]).mode, 'effects');
  const generated = build('Selected', [strike(40), lowerDefense(), message()]);
  assert.match(generated.lua, /ctx:damage\(40\)\n\s+if ctx:reached\(\) then\n\s+ctx:change_stat\(ctx\.target, ctx\.Stat\.Defense, -1\)/);
  const hit = playTurns(generated, [mon('Ally', ['check'])], [bystander('Foe')]);
  assert.deepEqual(hit.state.foes[0].stages, [['Defense', -1]]);
  const blocked = playTurns(generated, [mon('Ally', ['check'])], [mon('Foe', ['guard'])], 1, {
    extra: { 'guard.lua': guard },
    picks: (actor) => (actor.side === 'foes' ? 'guard' : 'check'),
  });
  assert.deepEqual(blocked.state.foes[0].stages, []);
  assert.equal(blocked.messages.filter((text) => text === 'Foe protected itself!').length, 2);
});

check('exact damage, multi-hit, weather and the rest of the plain blocks work inside logic', () => {
  const stack = [
    when(
      { type: 'mlab_weather_is', fields: { OP: 'IS', WEATHER: 'Rain' } },
      chain({ type: 'mlab_exact_damage', inputs: { AMOUNT: number(30) } }, { type: 'mlab_multi_hit', fields: { MIN: 2, MAX: 2, POWER: 10 } }),
      chain({ type: 'mlab_weather', fields: { WEATHER: 'Rain', TURNS: 5 } }, { type: 'mlab_confuse_target' }),
    ),
  ];
  const generated = build('Selected', stack);
  const first = playTurns(generated, [mon('Ally', ['check'])], [mon('Foe', ['splash'], { max_hp: 400, hp: 400 })]);
  assert.equal(first.state.weather.kind, 'Rain');
  assert.equal(first.state.foes[0].confused, true);
  const second = playTurns(generated, [mon('Ally', ['check'])], [mon('Foe', ['splash'], { max_hp: 400, hp: 400 })], 2);
  assert.ok(second.messages.includes('Hit 2 times!'));
  assert.ok(second.state.foes[0].hp < 400 - 30);
});

check('“for each foe” acts on every foe, each with its own check', () => {
  const each = {
    type: 'mlab_for_each',
    fields: { GROUP: 'foes' },
    inputs: { DO: { block: { type: 'mlab_status', fields: { WHO: 'each', STATUS: 'Paralyzed', CHANCE: 100 } } } },
  };
  const generated = build('AllOpponents', [each]);
  assert.match(generated.lua, /for _, pokemon in ipairs\(ctx\.foes\) do\n\s+if ctx:reached\(pokemon\) then/);
  const { state } = playTurns(generated, [mon('Ally', ['check'])], [bystander('Foe'), bystander('Other')]);
  assert.deepEqual(state.foes.map((foe) => foe.status), ['Paralyzed', 'Paralyzed']);
  assert.equal(state.allies[0].status, null);
});

check('“as a … move” changes the type of the hits inside it', () => {
  const asFire = { type: 'mlab_as_type', inputs: { TYPE: { shadow: { type: 'mlab_type', fields: { TYPE: 'Fire' } } }, DO: { block: strike(40) } } };
  const grass = () => [mon('Foe', ['splash'], { types: ['Grass'], max_hp: 400, hp: 400 })];
  const plain = playTurns(build('Selected', [strike(40), message()]), [mon('Ally', ['check'])], grass());
  const fire = playTurns(build('Selected', [asFire]), [mon('Ally', ['check'])], grass());
  assert.ok(fire.messages.includes("It's super effective!"));
  assert.ok(400 - fire.state.foes[0].hp > 400 - plain.state.foes[0].hp);

  // The “type” block of the palette says the same as the menu the socket comes with.
  const plugged = (block) => ({ ...asFire, inputs: { ...asFire.inputs, TYPE: { ...asFire.inputs.TYPE, block } } });
  const picked = build('Selected', [plugged({ type: 'mlab_type_choice', fields: { TYPE: 'Fire' } })]);
  assert.equal(workspace.getAllBlocks(false).find((block) => block.type === 'mlab_type_choice').toString(), 'type Fire');
  assert.equal(picked.lua, build('Selected', [asFire]).lua);
  // “type number” left the palette, but a move saved with it still opens and writes what it did.
  const old = build('Selected', [plugged({ type: 'mlab_type_by_number', inputs: { NUMBER: number(8) } })]);
  assert.deepEqual(old.problems, []);
  assert.match(old.lua, /local move_type = ctx:type_number\(8\)/);
});

check('a screen built from an effect and a rule halves physical damage, until it is ended', () => {
  const screen = {
    type: 'mlab_effect',
    fields: { TURNS: 5, SCOPE: 'user_side', NAME: 'reflect' },
    inputs: { RULES: { block: { type: 'mlab_rule_damage_taken', fields: { FACTOR: 0.5, CATEGORY: 'Physical' } } } },
  };
  const generated = build('User', [screen]);
  assert.match(generated.lua, /ctx:start_effect\(ctx\.user\.team, "reflect", \{turns = 5, rules = \{\n\s+\{kind = ctx\.Rule\.DamageTaken, factor = 0\.5, category = ctx\.Category\.Physical\},/);
  const foe = () => [mon('Foe', ['tackle'], { speed: 10 })];
  const open = playTurns(build('User', [message()]), [mon('Ally', ['check'])], foe());
  const walled = playTurns(generated, [mon('Ally', ['check'])], foe());
  const lost = (result) => 140 - result.state.allies[0].hp;
  assert.ok(lost(walled) > 0 && lost(walled) * 2 <= lost(open) + 1, `${lost(walled)} against ${lost(open)}`);
  assert.deepEqual(walled.state.sides.allies, [{ name: 'reflect', value: 1, turns: 4 }]);

  const breaker = build('Selected', [{ type: 'mlab_end_effect', fields: { NAME: 'reflect', SCOPE: 'target_side' } }, strike(40)]);
  assert.match(breaker.lua, /ctx:end_effect\(ctx\.target\.team, "reflect"\)/);
});

check('a mark left by one move is read by another', () => {
  const stomp = build('Selected', [
    when({ type: 'mlab_has_mark', fields: { SCOPE: 'target', NAME: 'minimized' } }, strike(80), strike(40)),
  ]);
  const minimize = `return { id="shrink", name="Shrink", type=Type.Normal, category=Category.Status, pp=10, target=Target.User,
    script=function(ctx) ctx:mark(ctx.user, "minimized") end }`;
  const run = (foeMove) =>
    playTurns(stomp, [mon('Ally', ['check'], { speed: 10 })], [mon('Foe', [foeMove], { max_hp: 400, hp: 400 })], 1, {
      extra: { 'shrink.lua': minimize },
      picks: (actor) => (actor.side === 'foes' ? foeMove : 'check'),
    });
  const small = 400 - run('shrink').state.foes[0].hp;
  const normal = 400 - run('splash').state.foes[0].hp;
  assert.ok(small > normal * 1.8, `${small} against ${normal}`);

  const counter = build('User', [
    { type: 'mlab_mark', fields: { SCOPE: 'user', NAME: 'stock' }, inputs: { VALUE: { shadow: { type: 'mlab_number', fields: { NUM: 1 } }, block: {
      type: 'mlab_arith', fields: { OP: 'ADD' }, inputs: { A: { block: { type: 'mlab_mark_value', fields: { SCOPE: 'user', NAME: 'stock' } } }, B: number(1) } } } } },
  ]);
  const { state } = playTurns(counter, [mon('Ally', ['check'])], [bystander('Foe')], 3);
  assert.deepEqual(state.allies[0].conditions, [{ name: 'stock', value: 3, turns: null }]);
});

check('a move can get ready at the start of the turn and react to what it was hit by', () => {
  workspace.clear();
  const state = workspaceState('Selected', [when({ type: 'mlab_compare', fields: { OP: 'GT' }, inputs: {
    A: { block: { type: 'mlab_hp', fields: { WHO: 'user', FACT: 'damage_taken' } } }, B: number(0) } }, { type: 'mlab_fail' }), strike(150)]);
  state.blocks.blocks.push({ type: 'mlab_on_turn_start', x: 400, y: 20, next: { block: { type: 'mlab_message', fields: { TEXT: '{user} is tightening its focus!' } } } });
  Blockly.serialization.workspaces.load(state, workspace);
  const generated = generate(workspace, { ...newMoveSheet('Check', 'check'), priority: -3 });
  assert.match(generated.lua, /on_turn_start = function\(ctx\)/);
  const hurt = playTurns(generated, [mon('Ally', ['check'])], [mon('Foe', ['tackle'])]);
  assert.equal(hurt.messages[0], 'Ally is tightening its focus!');
  assert.ok(hurt.messages.includes('But it failed!'));
  assert.equal(hurt.state.foes[0].hp, 140);
  const calm = playTurns(generated, [mon('Ally', ['check'])], [bystander('Foe')]);
  assert.ok(calm.state.foes[0].hp < 140);
});

check('hiding, and a hit that reaches the hiding place with double power', () => {
  const fly = build('Selected', [
    when({ type: 'mlab_first_turn' }, chain({ type: 'mlab_force_move', fields: { POLICY: 'SameTarget' }, inputs: { TURNS: number(2) } },
      { type: 'mlab_hide', fields: { HIDDEN: 'Air' } }, { type: 'mlab_stop' })),
    strike(90),
  ]);
  const gust = `return { id="gust", name="Gust", type=Type.Flying, category=Category.Special, pp=10, hits_hidden={Hidden.Air},
    script=function(ctx) local power = 40 if ctx.target.hidden == ctx.Hidden.Air then power = power * 2 end ctx:damage(power) end }`;
  const up = (foeMove) =>
    playTurns(fly, [mon('Ally', ['check'], { speed: 90, max_hp: 400, hp: 400 })], [mon('Foe', [foeMove])], 1, {
      extra: { 'gust.lua': gust },
      picks: (actor) => (actor.side === 'foes' ? foeMove : 'check'),
    });
  const missed = up('tackle');
  assert.equal(missed.state.allies[0].hidden_in, 'Air');
  assert.equal(missed.state.allies[0].hp, 400);
  assert.ok(missed.messages.includes('Ally avoided the attack!'));
  assert.ok(up('gust').state.allies[0].hp < 400);
});

check('items, types and money', () => {
  const thief = build('Selected', [strike(40), { type: 'mlab_steal_item' }, { type: 'mlab_payout', inputs: { AMOUNT: number(50) } }]);
  const { state } = playTurns(thief, [mon('Ally', ['check'])], [{ ...bystander('Foe'), item: 'oran_berry' }]);
  assert.equal(state.allies[0].item, 'oran_berry');
  assert.equal(state.foes[0].item, null);
  assert.equal(state.payout.allies, 50);

  const burnUp = build('Selected', [strike(60), { type: 'mlab_lose_type', fields: { WHO: 'user', TYPE: 'Fire' } }]);
  const after = playTurns(burnUp, [mon('Ally', ['check'], { types: ['Fire', 'Flying'] })], [bystander('Foe')]);
  assert.deepEqual(after.state.allies[0].types, ['Flying']);
});

check('a stack that reacts cannot hold blocks that attack', () => {
  workspace.clear();
  const state = workspaceState('Selected', [{ type: 'mlab_watch_hits' }, strike(20)]);
  state.blocks.blocks.push({ type: 'mlab_on_hit', x: 400, y: 20, next: { block: chain(
    when({ type: 'mlab_hit_was', fields: { KIND: 'contact' } }, { type: 'mlab_status', fields: { WHO: 'target', STATUS: 'Burned', CHANCE: 100 } }),
    strike(10),
  ) } });
  Blockly.serialization.workspaces.load(state, workspace);
  const generated = generate(workspace, newMoveSheet('Check', 'check'));
  assert.equal(generated.problems.length, 1);
  assert.match(generated.problems[0].message, /can’t run here/);
  // The reaction itself acts at once, with no check: the attacker is right there.
  assert.match(generated.lua, /if ctx\.hit\.contact then\n\s+ctx:apply_status\(ctx\.target, ctx\.Status\.Burned\)/);
});

check('blocks inside “ignoring protection, hiding and accuracy” get through to a protected Pokémon', () => {
  const sure = { type: 'mlab_sure', inputs: { DO: { block: chain(poison(), strike(40), { type: 'mlab_flinch', fields: { CHANCE: 100 } }) } } };
  const generated = build('Selected', [sure]);
  assert.doesNotMatch(generated.lua, /ctx:reached/);
  assert.match(generated.lua, /ctx:damage\(40, \{never_miss = true, ignore_protect = true, hits_hidden = true\}\)/);
  const guarded = { extra: { 'guard.lua': guard }, picks: (actor) => (actor.side === 'foes' ? 'guard' : 'check') };
  const through = playTurns(generated, [mon('Ally', ['check'])], [mon('Foe', ['guard'])], 1, guarded);
  assert.equal(through.state.foes[0].status, 'Poisoned');
  assert.ok(through.state.foes[0].hp < 140);
  // Outside the block the same three wait for the move to get through, and it does not.
  const held = build('Selected', [poison(), strike(40), { type: 'mlab_flinch', fields: { CHANCE: 100 } }, message()]);
  assert.match(held.lua, /if ctx:reached\(\) then\n\s+ctx:flinch_target\(1\)/);
  const stopped = playTurns(held, [mon('Ally', ['check'])], [mon('Foe', ['guard'])], 1, guarded);
  assert.equal(stopped.state.foes[0].status, null);
  assert.equal(stopped.state.foes[0].hp, 140);
});

check('a move used on “a chosen foe or ally” offers both, and can tell them apart', () => {
  const puff = build('AnyOther', [
    when(
      { type: 'mlab_state_is', fields: { WHO: 'target', STATE: 'ally' } },
      { type: 'mlab_restore', fields: { WHO: 'target', UNIT: 'PERCENT' }, inputs: { AMOUNT: number(50) } },
      strike(90),
    ),
  ]);
  assert.deepEqual(puff.problems, []);
  assert.match(puff.lua, /target = Target\.AnyOther,/);
  engine.setMoves({ ...originals, 'check.lua': puff.lua });
  const aimedAt = (side) => {
    const battle = engine.createBattle({ seed: 3, allies: [mon('Ally', ['check']), mon('Partner', ['sunny_day'], { hp: 40 })], foes: [bystander('Foe')] });
    try {
      for (let step = 0; step < 12; step += 1) {
        const result = battle.advance();
        if (result.status.kind === 'end' || result.turn >= 1) break;
        const { choices, prompt_id: promptId, actor } = result.status;
        const mine = choices.filter((choice) => choice.move_id === 'check');
        if (actor.side === 'allies' && actor.index === 0) {
          assert.deepEqual(mine.map((choice) => `${choice.target.side}:${choice.target.index}`).sort(), ['allies:1', 'foes:0']);
        }
        battle.respond(promptId, (mine.find((choice) => choice.target.side === side) ?? choices[0]).id);
      }
      return battle.state();
    } finally {
      battle.dispose();
    }
  };
  const healed = aimedAt('allies');
  assert.equal(healed.allies[1].hp, 110);
  assert.equal(healed.foes[0].hp, 140);
  const struck = aimedAt('foes');
  assert.equal(struck.allies[1].hp, 40);
  assert.ok(struck.foes[0].hp < 140);
});

check('the ready-made two-turn block says where the user hides', () => {
  const dig = { type: 'mlab_two_turn', fields: { MESSAGE: '{user} dug a hole!', POWER: 80, PLACE: 'Underground' }, extraState: { options: ['hidden'] } };
  const generated = build('Selected', [dig]);
  assert.equal(generated.mode, 'effects');
  assert.match(generated.lua, /semi_invulnerable = true, hidden = Hidden\.Underground,/);
  const { state } = playTurns(generated, [mon('Ally', ['check'])], [bystander('Foe')]);
  assert.equal(state.allies[0].hidden_in, 'Underground');
  // A place that is not named stays what it always was.
  const vanish = { ...dig, fields: { MESSAGE: '{user} vanished!', POWER: 80 } };
  assert.doesNotMatch(build('Selected', [vanish]).lua, /hidden = /);
});

check('“protect the user” works less often turn after turn, in a script as in a plain move', () => {
  const scripted = build('User', [protect(), message()]);
  assert.equal(scripted.mode, 'script');
  assert.equal(build('User', [protect()]).mode, 'effects');
  const outcomes = (generated) => {
    engine.setMoves({ ...originals, 'check.lua': generated.lua });
    let first = 0;
    let second = 0;
    for (let seed = 1; seed <= 30; seed += 1) {
      const battle = engine.createBattle({ seed, allies: [mon('Ally', ['check'])], foes: [bystander('Foe')] });
      try {
        for (let step = 0; step < 12; step += 1) {
          const result = battle.advance();
          const held = result.events.some((event) => event.kind === 'message' && event.text === 'Ally protected itself!');
          if (held && result.turn === 1) first += 1;
          if (held && result.turn === 2) second += 1;
          if (result.status.kind === 'end' || result.turn >= 2) break;
          const { choices, prompt_id: promptId } = result.status;
          battle.respond(promptId, (choices.find((choice) => choice.move_id === 'check') ?? choices[0]).id);
        }
      } finally {
        battle.dispose();
      }
    }
    return { first, second };
  };
  for (const generated of [scripted, build('User', [protect()])]) {
    const { first, second } = outcomes(generated);
    assert.equal(first, 30);
    // One chance in three the second time: about 10 of 30.
    assert.ok(second >= 3 && second <= 18, `held ${second} times out of 30 on the second turn`);
  }
  const endure = build('User', [when({ type: 'mlab_not', inputs: { A: { block: { type: 'mlab_repeat_works' } } } }, { type: 'mlab_fail' }), message()]);
  assert.match(endure.lua, /if not ctx:chance\(\(1 \/ 3\) \^ ctx\.user\.streak\) then/);
  assert.ok(playTurns(endure, [mon('Ally', ['check'])], [bystander('Foe')]).messages.includes('Hello!'));
});

check('a move can strike back with twice the damage of the physical hit its user just took', () => {
  const counter = build('Selected', [
    when({ type: 'mlab_not', inputs: { A: { block: { type: 'mlab_hit_was', fields: { KIND: 'Physical' } } } } }, { type: 'mlab_fail' }),
    { type: 'mlab_exact_damage', inputs: { AMOUNT: { shadow: { type: 'mlab_number', fields: { NUM: 1 } }, block: {
      type: 'mlab_arith', fields: { OP: 'MULTIPLY' }, inputs: { A: { block: { type: 'mlab_hp', fields: { WHO: 'user', FACT: 'last_hit_damage' } } }, B: number(2) } } } } },
  ]);
  assert.deepEqual(counter.problems, []);
  const slow = () => [mon('Ally', ['check'], { speed: 10, max_hp: 400, hp: 400 })];
  const hit = playTurns(counter, slow(), [mon('Foe', ['tackle'], { max_hp: 400, hp: 400 })], 1, { picks: (actor) => (actor.side === 'foes' ? 'tackle' : 'check') });
  const taken = 400 - hit.state.allies[0].hp;
  assert.ok(taken > 0);
  assert.equal(400 - hit.state.foes[0].hp, taken * 2);
  const idle = playTurns(counter, slow(), [bystander('Foe')]);
  assert.ok(idle.messages.includes('But it failed!'));
});

check('a power with a fraction is rounded down instead of stopping the battle', () => {
  const eruption = build('Selected', [
    strike({ type: 'mlab_arith', fields: { OP: 'DIVIDE' }, inputs: {
      A: { block: { type: 'mlab_arith', fields: { OP: 'MULTIPLY' }, inputs: { A: number(150), B: { block: { type: 'mlab_hp', fields: { WHO: 'user', FACT: 'hp' } } } } } },
      B: { block: { type: 'mlab_hp', fields: { WHO: 'user', FACT: 'max_hp' } } } } }),
  ]);
  assert.match(eruption.lua, /ctx:damage\(\(150 \* ctx\.user\.hp\) \/ ctx\.user\.max_hp\)/);
  const weak = playTurns(eruption, [mon('Ally', ['check'], { hp: 33 })], [mon('Foe', ['splash'], { max_hp: 400, hp: 400 })]);
  const strong = playTurns(eruption, [mon('Ally', ['check'])], [mon('Foe', ['splash'], { max_hp: 400, hp: 400 })]);
  assert.ok(weak.state.foes[0].hp < 400 && weak.state.foes[0].hp > strong.state.foes[0].hp);
});

/** Which foes a three-turn move used on the second foe hits on its first turn and on its later turns, over many seeds. */
function threeTurnTargets(policy) {
  const generated = build('Selected', [onFirstTurn(threeTurns(policy)), damage()]);
  assert.deepEqual(generated.problems, []);
  engine.setMoves({ ...originals, 'check.lua': generated.lua });
  const first = new Set();
  const later = new Set();
  const sturdy = (name) => ({ ...bystander(name), max_hp: 400, hp: 400 });
  for (let seed = 1; seed <= 16; seed += 1) {
    const battle = engine.createBattle({ seed, allies: [mon('Ally', ['check'])], foes: [sturdy('Foe'), sturdy('Other')] });
    try {
      for (let step = 0; step < 30; step += 1) {
        const result = battle.advance();
        for (const event of result.events) {
          if (event.kind === 'damage' && event.target.side === 'foes') (result.turn === 1 ? first : later).add(event.target.index);
        }
        if (result.status.kind === 'end' || result.turn >= 3) break;
        const { choices, prompt_id: promptId } = result.status;
        battle.respond(promptId, (choices.find((choice) => choice.move_id === 'check' && choice.target.index === 1) ?? choices[0]).id);
      }
    } finally {
      battle.dispose();
    }
  }
  return { first: [...first].sort(), later: [...later].sort() };
}

check('a multi-turn move “aimed at the same target” keeps hitting the foe it was used on', () => {
  assert.deepEqual(threeTurnTargets('SameTarget'), { first: [1], later: [1] });
});

check('a multi-turn move “aimed at a random foe” picks a foe again on each later turn', () => {
  assert.deepEqual(threeTurnTargets('RandomOpponent'), { first: [1], later: [0, 1] });
});

console.log(`\n${checks} checks passed.`);
