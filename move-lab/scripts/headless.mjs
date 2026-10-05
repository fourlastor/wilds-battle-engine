// Shared setup for the checks that build blocks without a browser: Blockly running headless, the
// editor's blocks defined, and the wasm engine loaded with the repo's move files.
//
// Import this first, then load the editor's modules with `await import(...)`: they import
// "blockly/core", which only resolves to the right build once the hook below is registered.
import { readFileSync, readdirSync } from 'node:fs';
import { registerHooks } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, '..');

// In Node, "blockly/core" resolves to a CommonJS build without named exports. The browser build
// has them and runs headless just as well, so point the editor's imports at that one.
const blocklyEsm = pathToFileURL(path.join(root, 'node_modules/blockly/blockly.mjs')).href;
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier === 'blockly/core') return { url: blocklyEsm, shortCircuit: true };
    return nextResolve(specifier, context);
  },
});

export const Blockly = await import('blockly/core');
// Blockly's own Node entry does this too: shadow blocks are serialized through XML helpers.
const { JSDOM } = await import('jsdom');
Blockly.utils.xml.injectDependencies(new JSDOM('<!DOCTYPE html>').window);
const { defineBlocks } = await import('../src/blocks/definitions.ts');
const { Engine } = await import('../src/engine/engine.ts');

defineBlocks();
export const engine = await Engine.load(pathToFileURL(path.join(root, 'public/engine')).href + '/');

/** The repo's move files, by file name. */
export const originals = {};
const movesDir = path.join(root, '../moves');
for (const file of readdirSync(movesDir).filter((name) => name.endsWith('.lua'))) {
  originals[file] = readFileSync(path.join(movesDir, file), 'utf8');
}
