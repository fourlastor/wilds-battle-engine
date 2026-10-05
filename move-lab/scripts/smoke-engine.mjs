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

engine._wbe_destroy(battle);
console.log(`ok: ${catalog.moves.length} moves loaded, Solar Beam dealt ${158 - result.foes[0].hp} damage over ${state.turn} turns`);
console.log(messages.map((message) => `  ${message}`).join('\n'));
