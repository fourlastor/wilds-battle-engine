// Thin wrapper around the wasm build of the battle engine (see ../../engine-wasm).
// Battles go through the engine's own C ABI; move files live in Emscripten's in-memory file system.

export type Side = 'allies' | 'foes';
export interface Ref {
  side: Side;
  index: number;
}

export type EngineEvent =
  | { kind: 'message'; text: string }
  | { kind: 'damage'; target: Ref; amount: number; hp: number }
  | { kind: 'heal'; target: Ref; amount: number; hp: number }
  | { kind: 'status'; target: Ref; status: string | null }
  | { kind: 'stat_change'; target: Ref; stat: string; stages: number }
  | { kind: 'weather'; weather: string | null }
  | { kind: 'fainted'; target: Ref }
  | { kind: 'end'; winner: Side | null };

export interface EngineChoice {
  id: number;
  kind: 'use_move';
  move_id: string;
  target: Ref;
}

export type EngineStatus =
  | { kind: 'awaiting'; prompt_id: number; actor: Ref; choices: EngineChoice[] }
  | { kind: 'end'; winner: Side | null };

export interface Advance {
  events: EngineEvent[];
  status: EngineStatus;
  turn: number;
}

export interface EngineMon {
  name: string;
  level: number;
  hp: number;
  max_hp: number;
  attack: number;
  defense: number;
  sp_attack: number;
  sp_defense: number;
  speed: number;
  types: string[];
  status: string | null;
  stages: [string, number][];
  confused: boolean;
  bound: boolean;
  protected: boolean;
  recharging: boolean;
  hidden: boolean;
  locked: boolean;
  boost: { type: string; multiplier: number } | null;
  moves: string[];
  move_pp: Record<string, number>;
}

export interface EngineState {
  turn: number;
  weather: { kind: string; turns_left: number } | null;
  allies: EngineMon[];
  foes: EngineMon[];
}

export interface EffectInfo {
  kind: string;
  [key: string]: unknown;
}

export interface MoveInfo {
  id: string;
  name: string;
  type: string;
  category: string;
  pp: number;
  target: string;
  accuracy: { kind: 'chance'; value: number } | { kind: 'always' } | { kind: 'ohko' };
  priority: number;
  fail_on_full_hp: boolean;
  scripted: boolean;
  effects: EffectInfo[];
}

/** One Pokémon in the JSON the engine's `wbe_create` reads. */
export interface SetupMon {
  name: string;
  moves: string[];
  level: number;
  max_hp: number;
  hp: number;
  attack: number;
  defense: number;
  sp_attack: number;
  sp_defense: number;
  speed: number;
  types: string[];
  status?: string;
  status_turns?: number;
}

export interface EngineSetup {
  seed: number;
  allies: SetupMon[];
  foes: SetupMon[];
}

export class EngineError extends Error {}

interface WasmModule {
  FS: {
    mkdir(path: string): void;
    readdir(path: string): string[];
    unlink(path: string): void;
    writeFile(path: string, data: string): void;
  };
  UTF8ToString(pointer: number): string;
  stringToNewUTF8(text: string): number;
  _free(pointer: number): void;
  _wbe_create(setup: number, movesDir: number): number;
  _wbe_destroy(battle: number): void;
  _wbe_advance(battle: number): number;
  _wbe_respond(battle: number, promptId: bigint, choiceId: number): number;
  _wbe_last_error(): number;
  _wbe_free_string(pointer: number): void;
  _mlab_catalog(movesDir: number): number;
  _mlab_state(battle: number): number;
}

const MOVES_DIR = '/moves';

export class Engine {
  private readonly wasm: WasmModule;
  /** Text the engine printed (Lua `print`, panics); kept for error reports. */
  readonly printed: string[] = [];

  private constructor(wasm: WasmModule) {
    this.wasm = wasm;
    wasm.FS.mkdir(MOVES_DIR);
  }

  /** Loads `wbe.js` + `wbe.wasm` from `baseUrl` (built by `npm run build:engine`). */
  static async load(baseUrl: string): Promise<Engine> {
    const url = new URL('wbe.js', baseUrl).href;
    const loaded = (await import(/* @vite-ignore */ url)) as { default: (options: object) => Promise<WasmModule> };
    const printed: string[] = [];
    const wasm = await loaded.default({
      print: (line: string) => printed.push(line),
      printErr: (line: string) => printed.push(line),
    });
    const engine = new Engine(wasm);
    engine.printed.push(...printed);
    return engine;
  }

  /** Replaces every move file. Keys are file names such as `tackle.lua`. */
  setMoves(files: Record<string, string>): void {
    const { FS } = this.wasm;
    for (const name of FS.readdir(MOVES_DIR)) {
      if (name !== '.' && name !== '..') FS.unlink(`${MOVES_DIR}/${name}`);
    }
    for (const [name, source] of Object.entries(files)) FS.writeFile(`${MOVES_DIR}/${name}`, source);
  }

  /** Loads the current move files the way the game does and lists them, or throws with the engine's message. */
  catalog(): MoveInfo[] {
    const result = JSON.parse(this.withStrings([MOVES_DIR], (dir) => this.take(this.wasm._mlab_catalog(dir)))!) as
      | { moves: MoveInfo[] }
      | { error: string };
    if ('error' in result) throw new EngineError(cleanError(result.error));
    return result.moves;
  }

  createBattle(setup: EngineSetup): Battle {
    const pointer = this.withStrings([JSON.stringify(setup), MOVES_DIR], (json, dir) => this.wasm._wbe_create(json, dir));
    if (!pointer) throw new EngineError(this.lastError());
    return new Battle(this, pointer);
  }

  /** @internal */
  advance(pointer: number): Advance {
    const raw = this.take(this.wasm._wbe_advance(pointer));
    if (raw === null) throw new EngineError(this.lastError());
    return JSON.parse(raw) as Advance;
  }

  /** @internal */
  respond(pointer: number, promptId: number, choiceId: number): void {
    if (!this.wasm._wbe_respond(pointer, BigInt(promptId), choiceId)) throw new EngineError(this.lastError());
  }

  /** @internal */
  state(pointer: number): EngineState {
    return JSON.parse(this.take(this.wasm._mlab_state(pointer))!) as EngineState;
  }

  /** @internal */
  destroy(pointer: number): void {
    this.wasm._wbe_destroy(pointer);
  }

  private lastError(): string {
    return cleanError(this.take(this.wasm._wbe_last_error()) || 'the engine stopped without a message');
  }

  private take(pointer: number): string | null {
    if (!pointer) return null;
    const value = this.wasm.UTF8ToString(pointer);
    this.wasm._wbe_free_string(pointer);
    return value;
  }

  private withStrings<T>(values: string[], run: (...pointers: number[]) => T): T {
    const pointers = values.map((value) => this.wasm.stringToNewUTF8(value));
    try {
      return run(...pointers);
    } finally {
      for (const pointer of pointers) this.wasm._free(pointer);
    }
  }
}

export class Battle {
  private readonly engine: Engine;
  private pointer: number;

  constructor(engine: Engine, pointer: number) {
    this.engine = engine;
    this.pointer = pointer;
  }

  advance(): Advance {
    return this.engine.advance(this.pointer);
  }

  respond(promptId: number, choiceId: number): void {
    this.engine.respond(this.pointer, promptId, choiceId);
  }

  state(): EngineState {
    return this.engine.state(this.pointer);
  }

  dispose(): void {
    if (this.pointer) this.engine.destroy(this.pointer);
    this.pointer = 0;
  }
}

/** Engine messages mention the in-memory path and Lua chunk names; trim them for people. */
function cleanError(message: string): string {
  return message
    .replace(/\/moves\//g, 'moves/')
    .replace(/^Lua: /, '')
    .replace(/\nstack traceback:[\s\S]*$/, '')
    .trim();
}
