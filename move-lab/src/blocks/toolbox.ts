// The palette: one always-open list of blocks, grouped under category labels.
import * as Blockly from 'blockly/core';

export const COLORS = {
  hat: '#173C2E',
  damage: '#C5372D',
  heal: '#1E7B4C',
  status: '#7343C2',
  stats: '#1F6BCB',
  field: '#0B7C86',
  turns: '#B4317B',
  logic: '#A55F00',
  info: '#FFD95A',
  text: '#4D5E56',
} as const;

type BlockEntry = { kind: 'block'; type: string; inputs?: Record<string, unknown>; fields?: Record<string, unknown>; extraState?: unknown };

const n = (value: number) => ({ shadow: { type: 'mlab_number', fields: { NUM: value } } });
const block = (type: string, extra: Omit<BlockEntry, 'kind' | 'type'> = {}): BlockEntry => ({ kind: 'block', type, ...extra });

export interface ToolboxCategory {
  id: string;
  name: string;
  color: string;
  blocks: BlockEntry[];
}

export const CATEGORIES: ToolboxCategory[] = [
  {
    id: 'damage',
    name: 'Damage',
    color: COLORS.damage,
    blocks: [
      block('mlab_damage', { inputs: { POWER: n(40) } }),
      block('mlab_fixed_damage'),
      block('mlab_level_damage'),
      block('mlab_ohko'),
      block('mlab_multi_hit'),
      block('mlab_recoil_max_hp'),
      block('mlab_faint_user'),
    ],
  },
  { id: 'heal', name: 'Healing', color: COLORS.heal, blocks: [block('mlab_heal')] },
  {
    id: 'status',
    name: 'Status',
    color: COLORS.status,
    blocks: [
      block('mlab_status', { fields: { CHANCE: 30 } }),
      block('mlab_confuse_target'),
      block('mlab_confuse_self'),
      block('mlab_flinch'),
      block('mlab_bind'),
      block('mlab_protect'),
    ],
  },
  { id: 'stats', name: 'Stats', color: COLORS.stats, blocks: [block('mlab_stats'), block('mlab_boost_next', { fields: { TYPE: 'Electric' } })] },
  { id: 'field', name: 'Field', color: COLORS.field, blocks: [block('mlab_weather')] },
  {
    id: 'turns',
    name: 'Turns',
    color: COLORS.turns,
    blocks: [
      block('mlab_force_move', { inputs: { TURNS: n(2) } }),
      block('mlab_break_sequence'),
      block('mlab_recharge'),
      block('mlab_watch_hits'),
      block('mlab_on_hit'),
      block('mlab_on_interrupt'),
      block('mlab_fail'),
      block('mlab_stop'),
      block('mlab_two_turn'),
      block('mlab_consecutive'),
    ],
  },
  {
    id: 'logic',
    name: 'Logic',
    color: COLORS.logic,
    blocks: [
      block('mlab_if'),
      block('mlab_if_else'),
      block('mlab_repeat', { inputs: { TIMES: n(3) } }),
      block('mlab_repeat_count'),
      block('mlab_compare', { inputs: { A: n(0), B: n(0) } }),
      block('mlab_and_or'),
      block('mlab_not'),
      block('mlab_arith', { inputs: { A: n(10), B: n(2) }, fields: { OP: 'MULTIPLY' } }),
      block('mlab_random', { inputs: { FROM: n(2), TO: n(3) } }),
    ],
  },
  {
    id: 'info',
    name: 'Battle info',
    color: COLORS.info,
    blocks: [
      block('mlab_status_is', { fields: { WHO: 'target' } }),
      block('mlab_weather_is'),
      block('mlab_hit_landed'),
      block('mlab_first_turn'),
      block('mlab_last_turn'),
      block('mlab_in_sequence'),
      block('mlab_hp'),
      block('mlab_turn'),
      block('mlab_last_damage'),
    ],
  },
  { id: 'text', name: 'Text', color: COLORS.text, blocks: [block('mlab_message'), block('mlab_announce'), block('mlab_splash')] },
];

/** The palette contents, optionally narrowed to blocks whose text contains `search`. */
export function toolboxDefinition(search = ''): Blockly.utils.toolbox.ToolboxDefinition {
  const needle = search.trim().toLowerCase();
  const contents: object[] = [];
  for (const category of CATEGORIES) {
    const blocks = needle ? category.blocks.filter((entry) => blockText(entry.type).includes(needle)) : category.blocks;
    if (blocks.length === 0) continue;
    contents.push({ kind: 'label', text: category.name, 'web-class': `mlab-label mlab-label-${category.id}` });
    contents.push(...blocks);
    contents.push({ kind: 'sep', gap: 28 });
  }
  if (contents.length === 0) contents.push({ kind: 'label', text: 'No block matches', 'web-class': 'mlab-label' });
  return { kind: 'flyoutToolbox', contents } as Blockly.utils.toolbox.ToolboxDefinition;
}

const textCache = new Map<string, string>();

/** The words on a block, for the palette search. */
function blockText(type: string): string {
  let text = textCache.get(type);
  if (text === undefined) {
    const workspace = new Blockly.Workspace();
    try {
      text = workspace.newBlock(type).toString().toLowerCase();
    } catch {
      text = type;
    }
    workspace.dispose();
    textCache.set(type, text);
  }
  return text;
}

export const STYLE_COLORS: Record<string, string> = {
  hat_blocks: COLORS.hat,
  damage_blocks: COLORS.damage,
  heal_blocks: COLORS.heal,
  status_blocks: COLORS.status,
  stats_blocks: COLORS.stats,
  field_blocks: COLORS.field,
  turn_blocks: COLORS.turns,
  logic_blocks: COLORS.logic,
  info_blocks: COLORS.info,
  text_blocks: COLORS.text,
};
