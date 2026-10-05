// Shared vocabulary: the names here are the engine's own (see crates/wilds-battle-engine/src/lua_symbols.rs).

export const TYPES = [
  'Normal', 'Fighting', 'Flying', 'Poison', 'Ground', 'Rock', 'Bug', 'Ghost', 'Steel',
  'Fire', 'Water', 'Grass', 'Electric', 'Psychic', 'Ice', 'Dragon', 'Dark', 'Fairy',
] as const;

export const TYPE_COLORS: Record<string, string> = {
  Normal: '#7C8272', Fighting: '#B4532A', Flying: '#5E7FCB', Poison: '#8E4BB0', Ground: '#A5802F',
  Rock: '#857A4F', Bug: '#6F8C17', Ghost: '#5D4E8C', Steel: '#5F7C8C', Fire: '#D5552B',
  Water: '#2F7FD0', Grass: '#2F8F46', Electric: '#B58900', Psychic: '#B8367A', Ice: '#2E97A6',
  Dragon: '#4A55C9', Dark: '#4A4039', Fairy: '#BD5C9B', None: '#7C8272',
};

export const CATEGORIES = ['Physical', 'Special', 'Status'] as const;
export type Category = (typeof CATEGORIES)[number];

export const STAT_LABELS: Record<string, string> = {
  Attack: 'Attack', Defense: 'Defense', SpAttack: 'Sp. Atk', SpDefense: 'Sp. Def',
  Speed: 'Speed', Accuracy: 'Accuracy', Evasion: 'Evasion',
};
export const STAT_SHORT: Record<string, string> = {
  Attack: 'ATK', Defense: 'DEF', SpAttack: 'SPA', SpDefense: 'SPD', Speed: 'SPE', Accuracy: 'ACC', Evasion: 'EVA',
};

/** Statuses a Pokémon can start a battle with (the engine's `Status`). */
export const STATUSES = ['Poisoned', 'BadlyPoisoned', 'Burned', 'Paralyzed', 'Asleep', 'Frozen'] as const;
export const STATUS_LABELS: Record<string, string> = {
  Poisoned: 'Poisoned', BadlyPoisoned: 'Badly poisoned', Burned: 'Burned',
  Paralyzed: 'Paralyzed', Asleep: 'Asleep', Frozen: 'Frozen',
};
export const STATUS_SHORT: Record<string, string> = {
  Poisoned: 'PSN', BadlyPoisoned: 'TOX', Burned: 'BRN', Paralyzed: 'PAR', Asleep: 'SLP', Frozen: 'FRZ',
};

export type AccuracyRule = { kind: 'chance'; percent: number } | { kind: 'always' } | { kind: 'ohko' };

/** Everything about a move that is not a block. */
export interface MoveSheet {
  /** File name and move id: moves/<id>.lua */
  id: string;
  name: string;
  type: string;
  category: Category;
  pp: number;
  accuracy: AccuracyRule;
  priority: number;
  failOnFullHp: boolean;
  usableWhileAsleep: boolean;
}

export interface MoveDoc {
  uid: string;
  sheet: MoveSheet;
  /** Blockly workspace state. */
  blocks: unknown;
  /** True while the id still follows the name (new moves). Opened built-ins keep their id. */
  idFollowsName: boolean;
  /** Last generated file text, kept so other moves can be loaded without opening them. */
  lua: string;
  /** Whether `lua` loaded cleanly in the engine the last time it was checked. */
  valid: boolean;
}

export interface Stats {
  max_hp: number;
  attack: number;
  defense: number;
  sp_attack: number;
  sp_defense: number;
  speed: number;
}

export interface MonSetup {
  species: string;
  name: string;
  level: number;
  types: string[];
  status: string | null;
  hp: number;
  stats: Stats;
  /** Move ids. For the first Pokémon on your side the move being edited is added in front. */
  moves: string[];
}

export type FoePolicy = 'first' | 'random' | 'manual';
export type OnEdit = 'replay' | 'restart' | 'manual';

export interface BattleSetup {
  seed: number;
  allies: MonSetup[];
  foes: MonSetup[];
  foePolicy: FoePolicy;
  onEdit: OnEdit;
}

export function slug(name: string): string {
  const value = name.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_+|_+$/g, '');
  return value || 'my_move';
}

export function uid(): string {
  return Math.random().toString(36).slice(2, 10) + Date.now().toString(36);
}

export function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
