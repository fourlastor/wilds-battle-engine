// Checks that the blocks do what their words and the "All blocks" page say:
// every block is explained and writes at least one line, and a block that says "the user" acts
// on the user in the real engine, whoever the move is aimed at.
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

check('a block that only works in plain moves is told what made the move a script', () => {
  const withMessage = build('Selected', [poison(), message()]).problems;
  assert.equal(withMessage.length, 1);
  assert.match(withMessage[0].message, /only works in plain moves for now\. This move is a script because of “show message”\./);
  const everyone = build('AllOthers', [sunlight()]).problems;
  assert.equal(everyone.length, 1);
  assert.match(everyone[0].message, /because it is used on “everyone else”\./);
});

console.log(`\n${checks} checks passed.`);
