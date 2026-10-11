// Loads the wasm engine in Node and plays a short battle with the repo's move files.
// Run with: npm run test:engine
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const engineDir = path.join(here, '../public/engine');
const movesDir = path.join(here, '../../moves');

const { default: createEngine } = await import(pathToFileURL(path.join(engineDir, 'wbe.js')).href);
const printed = [];
const engine = await createEngine({ print: (line) => printed.push(line), printErr: (line) => printed.push(line) });

const text = (pointer) => {
  if (!pointer) return null;
  const value = engine.UTF8ToString(pointer);
  engine._wbe_free_string(pointer);
  return value;
};
const withStrings = (values, run) => {
  const pointers = values.map((value) => engine.stringToNewUTF8(value));
  try {
    return run(...pointers);
  } finally {
    pointers.forEach((pointer) => engine._free(pointer));
  }
};
const lastError = () => text(engine._wbe_last_error());

engine.FS.mkdir('/moves');
const files = readdirSync(movesDir).filter((file) => file.endsWith('.lua'));
for (const file of files) engine.FS.writeFile(`/moves/${file}`, readFileSync(path.join(movesDir, file), 'utf8'));

// The catalog lists every move file plus the engine's own Struggle.
const catalog = JSON.parse(withStrings(['/moves'], (dir) => text(engine._mlab_catalog(dir))));
assert.equal(catalog.error, undefined, catalog.error);
assert.equal(catalog.moves.length, files.length + 1);
const tackle = catalog.moves.find((move) => move.id === 'tackle');
assert.deepEqual(tackle.effects, [{ kind: 'Damage', power: 40, recoil: null, drain: null, high_crit: false, always_crit: false }]);
assert.equal(catalog.moves.find((move) => move.id === 'solar_beam').scripted, true);

// A scripted move that takes two turns, against a move that does nothing.
const setup = {
  seed: 7,
  allies: [{ name: 'Sunflora', level: 30, max_hp: 89, sp_attack: 72, speed: 27, types: ['Grass'], moves: ['solar_beam'] }],
  foes: [{ name: 'Wobbuffet', level: 30, max_hp: 158, sp_defense: 44, speed: 29, types: ['Psychic'], moves: ['splash'] }],
};
const battle = withStrings([JSON.stringify(setup), '/moves'], (json, dir) => engine._wbe_create(json, dir));
assert.notEqual(battle, 0, lastError());

const messages = [];
let result;
for (let step = 0; step < 12; step += 1) {
  const raw = text(engine._wbe_advance(battle));
  assert.notEqual(raw, null, lastError());
  result = JSON.parse(raw);
  for (const event of result.events) if (event.kind === 'message') messages.push(event.text);
  if (result.status.kind === 'end' || result.turn >= 2) break;
  const choice = result.status.choices[0];
  assert.ok(engine._wbe_respond(battle, BigInt(result.status.prompt_id), choice.id), lastError());
}

assert.ok(messages.includes('Sunflora took in sunlight!'), messages.join(' | '));
assert.ok(messages.includes('Sunflora used Solar Beam!'), messages.join(' | '));
assert.ok(result.foes[0].hp < 158, 'Solar Beam should have dealt damage');

const state = JSON.parse(text(engine._mlab_state(battle)));
assert.equal(state.turn, 2);
assert.equal(state.allies[0].name, 'Sunflora');
assert.equal(state.foes[0].hp, result.foes[0].hp);

// A broken move file is reported, not fatal.
engine.FS.writeFile('/moves/broken.lua', 'return { id="broken", name="Broken", type=Type.Normal }');
const broken = JSON.parse(withStrings(['/moves'], (dir) => text(engine._mlab_catalog(dir))));
assert.match(broken.error, /broken\.lua/);
engine.FS.unlink('/moves/broken.lua');

engine._wbe_destroy(battle);
console.log(`ok: ${catalog.moves.length} moves loaded, Solar Beam dealt ${158 - result.foes[0].hp} damage over ${state.turn} turns`);
console.log(messages.map((message) => `  ${message}`).join('\n'));

// Bench setup and replacement prompts cross both the C ABI and Move Lab's state API.
const benchBattle = withStrings([JSON.stringify({
  allies: [{ name: 'Fainted', hp: 0, moves: ['splash'] }, { name: 'Reserve', moves: ['splash'] }],
  foes: [{ name: 'Foe', moves: ['splash'] }], active: { allies: [0], foes: [0] },
}), '/moves'], (json, dir) => engine._wbe_create(json, dir));
assert.notEqual(benchBattle, 0, lastError());
const replacement = JSON.parse(text(engine._wbe_advance(benchBattle)));
assert.equal(replacement.status.reason, 'replacement');
assert.equal(replacement.status.choices[0].kind, 'switch');
assert.equal(engine._wbe_respond(benchBattle, BigInt(replacement.status.prompt_id), replacement.status.choices[0].id), 1);
const switched = JSON.parse(text(engine._wbe_advance(benchBattle)));
assert.equal(switched.status.actor.index, 1);
assert.deepEqual(switched.active.allies, [1]);
assert.ok(switched.events.some(event => event.kind === 'switch' && event.to.index === 1));
assert.deepEqual(JSON.parse(text(engine._mlab_state(benchBattle))).active.allies, [1]);
engine._wbe_destroy(benchBattle);
console.log('ok: bench replacement choices and active slots round-trip through wasm');

const pivotBattle = withStrings([JSON.stringify({
  allies: [{ name: 'Pivot', max_hp: 1000, speed: 200, moves: ['u-turn'] }, { name: 'Reserve', max_hp: 1000, moves: ['splash'] }],
  foes: [{ name: 'Foe', max_hp: 1000, speed: 50, moves: ['tackle'] }], active: { allies: [0], foes: [0] },
}), '/moves'], (json, dir) => engine._wbe_create(json, dir));
assert.notEqual(pivotBattle, 0, lastError());
for (let i = 0; i < 2; i++) {
  const step = JSON.parse(text(engine._wbe_advance(pivotBattle)));
  const move = step.status.choices.find(choice => choice.kind === 'use_move');
  assert.ok(engine._wbe_respond(pivotBattle, BigInt(step.status.prompt_id), move.id), lastError());
}
const pivot = JSON.parse(text(engine._wbe_advance(pivotBattle)));
assert.equal(pivot.status.reason, 'pivot');
assert.equal(pivot.turn, 1);
assert.equal(pivot.allies[1].hp, 1000);
const repeated = JSON.parse(text(engine._wbe_advance(pivotBattle)));
assert.equal(repeated.status.prompt_id, pivot.status.prompt_id);
assert.deepEqual(repeated.events, []);
assert.ok(engine._wbe_respond(pivotBattle, BigInt(pivot.status.prompt_id), pivot.status.choices[0].id));
const resumed = JSON.parse(text(engine._wbe_advance(pivotBattle)));
assert.equal(resumed.turn, 1);
assert.equal(resumed.status.reason, 'action');
assert.equal(resumed.allies[0].move_pp['u-turn'], 19);
assert.ok(resumed.allies[1].hp < 1000);
engine._wbe_destroy(pivotBattle);
console.log('ok: a pivot pauses for a host choice and resumes the same turn');

const carryBattle = withStrings([JSON.stringify({
  allies: [{ name: 'Carrier', max_hp: 1000, speed: 200, item: 'power-herb', moves: ['sky-drop'] }],
  foes: [{ name: 'Carried', max_hp: 1000, moves: ['splash'] }, { name: 'Reserve', moves: ['splash'] }],
  active: { allies: [0], foes: [0] },
}), '/moves'], (json, dir) => engine._wbe_create(json, dir));
assert.notEqual(carryBattle, 0, lastError());
for (let i = 0; i < 2; i++) {
  const step = JSON.parse(text(engine._wbe_advance(carryBattle)));
  const move = step.status.choices.find(choice => choice.kind === 'use_move');
  assert.ok(engine._wbe_respond(carryBattle, BigInt(step.status.prompt_id), move.id));
}
const lifted = JSON.parse(text(engine._wbe_advance(carryBattle)));
assert.deepEqual(lifted.allies[0].carrying, { side: 'foes', index: 0 });
assert.deepEqual(lifted.foes[0].carried_by, { side: 'allies', index: 0 });
assert.equal(lifted.allies[0].item, 'power-herb');
assert.ok(lifted.status.choices.every(choice => choice.kind === 'use_move'));
const carriedState = JSON.parse(text(engine._mlab_state(carryBattle)));
assert.deepEqual(carriedState.foes[0].carried_by, lifted.foes[0].carried_by);
assert.ok(engine._wbe_respond(carryBattle, BigInt(lifted.status.prompt_id), lifted.status.choices[0].id));
const dropped = JSON.parse(text(engine._wbe_advance(carryBattle)));
assert.equal(dropped.allies[0].carrying, null);
assert.equal(dropped.foes[0].carried_by, null);
assert.ok(dropped.foes[0].hp < 1000);
assert.equal(dropped.allies[0].move_pp['sky-drop'], 9);
engine._wbe_destroy(carryBattle);
console.log('ok: Sky Drop carries and releases both participants through wasm');

const substituteBattle = withStrings([JSON.stringify({
  allies: [{ name: 'Doll', max_hp: 400, speed: 200, moves: ['substitute'] }],
  foes: [{ name: 'Attacker', max_hp: 400, speed: 50, moves: ['dragon_rage'] }],
}), '/moves'], (json, dir) => engine._wbe_create(json, dir));
assert.notEqual(substituteBattle, 0, lastError());
for (let i = 0; i < 2; i++) {
  const step = JSON.parse(text(engine._wbe_advance(substituteBattle)));
  assert.ok(engine._wbe_respond(substituteBattle, BigInt(step.status.prompt_id), step.status.choices[0].id));
}
const doll = JSON.parse(text(engine._wbe_advance(substituteBattle)));
assert.equal(doll.allies[0].hp, 300);
assert.equal(doll.allies[0].substitute_hp, 60);
assert.ok(doll.events.some(event => event.kind === 'substitute' && event.amount === 40 && event.hp === 60));
assert.equal(JSON.parse(text(engine._mlab_state(substituteBattle))).allies[0].substitute_hp, 60);
engine._wbe_destroy(substituteBattle);
console.log('ok: Substitute HP and absorbed damage round-trip through wasm');
