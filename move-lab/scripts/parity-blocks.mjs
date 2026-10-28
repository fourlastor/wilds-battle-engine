// Rebuilds every block-editable built-in move and checks that the generated Lua behaves exactly like
// the original file: same events, same final state, over many seeded battles in the real engine.
// Run with: npm run test:blocks
import { Blockly, engine, originals } from './headless.mjs';

const { generate } = await import('../src/blocks/generate.ts');
const { blocksFromInfo, canOpen, sheetFromInfo } = await import('../src/blocks/library.ts');

engine.setMoves(originals);
const catalog = engine.catalog().filter((move) => move.id !== 'struggle');
const editable = catalog.filter(canOpen);
const luaOnly = catalog.filter((move) => !canOpen(move));
// Keep the original authored block recipes covered as the Lua catalog grows.
for (const id of ['charge', 'dream-eater', 'explosion', 'facade', 'false-swipe', 'hyper-beam',
  'morning-sun', 'rage', 'snore', 'solar-beam', 'thrash', 'triple-kick']) {
  if (!editable.some((move) => move.id === id)) throw new Error(`Missing block recipe: ${id}`);
}
// A battle loads only its moves and the fixture moves; unrelated files cannot affect these scenarios.
const fixtureIds = ['tackle', 'splash', 'headbutt', 'sunny-day', 'sandstorm'];
const filename = (id) => `${id.replaceAll('_', '-')}.lua`;

const mon = (name, moves, extra = {}) => ({
  name, moves, level: 40, max_hp: 140, hp: 140, attack: 70, defense: 60, sp_attack: 75, sp_defense: 65, speed: 50, types: ['Normal'], ...extra,
});

/** Situations that reach the different branches of the built-in scripts. */
function scenarios(move) {
  const cases = [
    { name: 'plain', allies: [mon('Ally', [move])], foes: [mon('Foe', ['tackle'], { speed: 60 })] },
    { name: 'user hurt and burned, target asleep', allies: [mon('Ally', [move], { hp: 50, status: 'Burned' })], foes: [mon('Foe', ['splash'], { status: 'Asleep', status_turns: 5 })] },
    { name: 'user asleep', allies: [mon('Ally', [move], { status: 'Asleep', status_turns: 4 })], foes: [mon('Foe', ['tackle'])] },
    { name: 'user poisoned', allies: [mon('Ally', [move], { status: 'Poisoned', hp: 90 })], foes: [mon('Foe', ['headbutt'])] },
    { name: 'sunny', allies: [mon('Ally', ['sunny-day', move], { hp: 60 })], foes: [mon('Foe', ['tackle'])], opener: 'sunny-day' },
    { name: 'sandstorm', allies: [mon('Ally', ['sandstorm', move], { hp: 60 })], foes: [mon('Foe', ['tackle'])], opener: 'sandstorm' },
    { name: 'two on two', allies: [mon('Ally', [move]), mon('Partner', ['tackle'], { speed: 30 })], foes: [mon('Foe', ['tackle'], { speed: 70 }), mon('Other', ['headbutt'], { speed: 20 })] },
    { name: 'used by the foe', allies: [mon('Ally', ['tackle', 'headbutt'])], foes: [mon('Foe', [move], { speed: 70 })] },
    { name: 'frail target', allies: [mon('Ally', [move], { attack: 200, sp_attack: 200 })], foes: [mon('Foe', ['tackle'], { max_hp: 30, hp: 30 })] },
  ];
  if (catalog.find((entry) => entry.id === move)?.flags.includes('charge')) {
    cases.push({ name: 'Power Herb', allies: [mon('Ally', [move], { item: 'power-herb' })], foes: [mon('Foe', ['splash'])] });
  }
  return cases;
}

/** Plays a battle and returns everything the engine reported, with marker messages removed. */
function play(files, scenario, move, seed) {
  engine.setMoves(files);
  let battle;
  try {
    battle = engine.createBattle({ seed, allies: scenario.allies, foes: scenario.foes });
  } catch (error) {
    return [`create failed: ${error.message}`];
  }
  const record = [];
  let prompts = 0;
  try {
    for (let step = 0; step < 80; step += 1) {
      const result = battle.advance();
      const events = result.events.filter((event) => !(event.kind === 'message' && event.text.startsWith('\u0001')));
      record.push(JSON.stringify(events), JSON.stringify(battle.state()));
      if (result.status.kind === 'end' || result.turn >= 14) break;
      const { choices, prompt_id: promptId } = result.status;
      prompts += 1;
      const opener = prompts === 1 && scenario.opener ? choices.find((choice) => choice.move_id === scenario.opener) : undefined;
      // Vary the target a little so moves are aimed at both foes.
      const preferred = choices.filter((choice) => choice.move_id === move);
      const choice = opener ?? preferred[(seed + prompts) % (preferred.length || 1)] ?? choices[(seed + prompts) % choices.length];
      battle.respond(promptId, choice.id);
    }
  } catch (error) {
    record.push(`error: ${error.message}`);
  }
  battle.dispose();
  return record;
}

function firstDifference(a, b) {
  for (let index = 0; index < Math.max(a.length, b.length); index += 1) {
    if (a[index] !== b[index]) return `step ${index}:\n      original:  ${String(a[index]).slice(0, 400)}\n      generated: ${String(b[index]).slice(0, 400)}`;
  }
  return '';
}

let failures = 0;
let battles = 0;
let crashed = 0;
const workspace = new Blockly.Workspace();

for (const info of editable) {
  workspace.clear();
  Blockly.serialization.workspaces.load(blocksFromInfo(info), workspace);
  const sheet = sheetFromInfo(info);
  const generated = generate(workspace, sheet);
  if (generated.problems.length) {
    console.log(`FAIL ${info.id}: ${generated.problems.map((problem) => problem.message).join('; ')}`);
    failures += 1;
    continue;
  }
  if (generated.mode !== (info.scripted ? 'script' : 'effects')) {
    console.log(`FAIL ${info.id}: expected a ${info.scripted ? 'script' : 'plain move'}, got ${generated.mode}`);
    failures += 1;
    continue;
  }

  // Self-check: PARITY_BREAK=tackle changes that move's first number, which must make it fail.
  if (process.env.PARITY_BREAK === info.id) {
    generated.lua = generated.lua.replace(/= (\d+)/, (_, digits) => `= ${Number(digits) + 1}`);
    generated.traced = generated.traced.replace(/= (\d+)/, (_, digits) => `= ${Number(digits) + 1}`);
  }

  const file = filename(info.id);
  const baseline = Object.fromEntries([...fixtureIds, info.id].map((id) => [filename(id), originals[filename(id)]]));
  let problem = '';
  for (const [label, source] of [['generated', generated.lua], ['traced', generated.traced]]) {
    const files = { ...baseline, [file]: source };
    for (const scenario of scenarios(info.id)) {
      for (let seed = 1; seed <= 8 && !problem; seed += 1) {
        battles += 1;
        const expected = play(baseline, scenario, info.id, seed);
        if (expected.some((entry) => entry.startsWith('create failed') || entry.startsWith('error:'))) crashed += 1;
        const actual = play(files, scenario, info.id, seed);
        const difference = firstDifference(expected, actual);
        if (difference) problem = `${label} file, "${scenario.name}", seed ${seed}, ${difference}`;
      }
    }
  }
  if (problem) {
    failures += 1;
    console.log(`FAIL ${info.id}: ${problem}\n--- generated ---\n${generated.lua}`);
  } else {
    console.log(`ok   ${info.id} (${generated.mode})`);
  }
}

console.log(`\n${editable.length - failures}/${editable.length} block-editable moves match their original files over ${battles} battles.`);
console.log(`${luaOnly.length} Lua-only moves have no authored block recipe and are covered by the Rust catalog tests.`);
if (crashed) console.log(`${crashed} of those battles stopped with an engine error in the original files too.`);
process.exit(failures ? 1 : 0);
