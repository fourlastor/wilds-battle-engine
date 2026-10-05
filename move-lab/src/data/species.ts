// A few Pokémon to fill the setup form quickly. The engine only needs names, types and stats,
// so every number stays editable and "Custom" works for anything not listed here.
import type { MonSetup, Stats } from '../model.ts';

export interface Species {
  name: string;
  types: string[];
  /** Base HP, Attack, Defense, Sp. Atk, Sp. Def, Speed. */
  base: [number, number, number, number, number, number];
  /** Kilograms. */
  weight: number;
}

export const SPECIES: Species[] = [
  { name: 'Bulbasaur', types: ['Grass', 'Poison'], base: [45, 49, 49, 65, 65, 45], weight: 6.9 },
  { name: 'Charmander', types: ['Fire'], base: [39, 52, 43, 60, 50, 65], weight: 8.5 },
  { name: 'Squirtle', types: ['Water'], base: [44, 48, 65, 50, 64, 43], weight: 9 },
  { name: 'Pikachu', types: ['Electric'], base: [35, 55, 40, 50, 50, 90], weight: 6 },
  { name: 'Geodude', types: ['Rock', 'Ground'], base: [40, 80, 100, 30, 30, 20], weight: 20 },
  { name: 'Gastly', types: ['Ghost', 'Poison'], base: [30, 35, 30, 100, 35, 80], weight: 0.1 },
  { name: 'Machop', types: ['Fighting'], base: [70, 80, 50, 35, 35, 35], weight: 19.5 },
  { name: 'Mankey', types: ['Fighting'], base: [40, 80, 35, 35, 45, 70], weight: 28 },
  { name: 'Abra', types: ['Psychic'], base: [25, 20, 15, 105, 55, 90], weight: 19.5 },
  { name: 'Slowpoke', types: ['Water', 'Psychic'], base: [90, 65, 65, 40, 40, 15], weight: 36 },
  { name: 'Eevee', types: ['Normal'], base: [55, 55, 50, 45, 65, 55], weight: 6.5 },
  { name: 'Snorlax', types: ['Normal'], base: [160, 110, 65, 65, 110, 30], weight: 460 },
  { name: 'Dratini', types: ['Dragon'], base: [41, 64, 45, 50, 50, 50], weight: 3.3 },
  { name: 'Magikarp', types: ['Water'], base: [20, 10, 55, 15, 20, 80], weight: 10 },
  { name: 'Chikorita', types: ['Grass'], base: [45, 49, 65, 49, 65, 45], weight: 6.4 },
  { name: 'Cyndaquil', types: ['Fire'], base: [39, 52, 43, 60, 50, 65], weight: 7.9 },
  { name: 'Totodile', types: ['Water'], base: [50, 65, 64, 44, 48, 43], weight: 9.5 },
  { name: 'Furret', types: ['Normal'], base: [85, 76, 64, 45, 55, 90], weight: 32.5 },
  { name: 'Hoothoot', types: ['Normal', 'Flying'], base: [60, 30, 30, 36, 56, 50], weight: 21.2 },
  { name: 'Mareep', types: ['Electric'], base: [55, 40, 40, 65, 45, 35], weight: 7.8 },
  { name: 'Marill', types: ['Water', 'Fairy'], base: [70, 20, 50, 20, 50, 40], weight: 8.5 },
  { name: 'Sudowoodo', types: ['Rock'], base: [70, 100, 115, 30, 65, 30], weight: 38 },
  { name: 'Sunflora', types: ['Grass'], base: [75, 75, 55, 105, 85, 30], weight: 8.5 },
  { name: 'Wobbuffet', types: ['Psychic'], base: [190, 33, 58, 33, 58, 33], weight: 28.5 },
  { name: 'Umbreon', types: ['Dark'], base: [95, 65, 110, 60, 130, 65], weight: 27 },
  { name: 'Heracross', types: ['Bug', 'Fighting'], base: [80, 125, 75, 40, 95, 85], weight: 54 },
  { name: 'Sneasel', types: ['Dark', 'Ice'], base: [55, 95, 55, 35, 75, 115], weight: 28 },
  { name: 'Swinub', types: ['Ice', 'Ground'], base: [50, 50, 40, 30, 30, 50], weight: 6.5 },
  { name: 'Skarmory', types: ['Steel', 'Flying'], base: [65, 80, 140, 40, 70, 70], weight: 50.5 },
  { name: 'Miltank', types: ['Normal'], base: [95, 80, 105, 40, 70, 100], weight: 75.5 },
  { name: 'Larvitar', types: ['Rock', 'Ground'], base: [50, 64, 50, 45, 50, 41], weight: 72 },
  { name: 'Togepi', types: ['Fairy'], base: [35, 20, 65, 40, 65, 20], weight: 1.5 },
];

/** What a Pokémon weighs when nothing says otherwise. */
export const DEFAULT_WEIGHT = 30;

export function findSpecies(name: string): Species | undefined {
  return SPECIES.find((species) => species.name === name);
}

/** Stats at a level, with middling individual values and no training. */
export function statsFor(species: Species, level: number): Stats {
  const scaled = (base: number) => Math.floor(((2 * base + 15) * level) / 100);
  const [hp, attack, defense, spAttack, spDefense, speed] = species.base;
  return {
    max_hp: scaled(hp) + level + 10,
    attack: scaled(attack) + 5,
    defense: scaled(defense) + 5,
    sp_attack: scaled(spAttack) + 5,
    sp_defense: scaled(spDefense) + 5,
    speed: scaled(speed) + 5,
  };
}

export function monFor(name: string, level: number, moves: string[]): MonSetup {
  const species = findSpecies(name);
  const stats = species ? statsFor(species, level) : { max_hp: 100, attack: 50, defense: 50, sp_attack: 50, sp_defense: 50, speed: 50 };
  return {
    species: species ? name : 'Custom', name, level, types: species ? [...species.types] : ['Normal'], status: null, hp: stats.max_hp, stats, moves,
    weight: species?.weight ?? DEFAULT_WEIGHT,
  };
}
