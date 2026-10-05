// The palette: one always-open list of blocks, grouped under category labels.
import * as Blockly from 'blockly/core';

export const COLORS = {
  hat: '#173C2E',
  damage: '#C5372D',
  heal: '#1E7B4C',
  status: '#7343C2',
  stats: '#1F6BCB',
  field: '#0B7C86',
  effects: '#5B7A0A',
  turns: '#B4317B',
  logic: '#A55F00',
  info: '#FFD95A',
  traits: '#34506E',
  text: '#4D5E56',
} as const;

type BlockEntry = { kind: 'block'; type: string; inputs?: Record<string, unknown>; fields?: Record<string, unknown>; extraState?: unknown };

const n = (value: number) => ({ shadow: { type: 'mlab_number', fields: { NUM: value } } });
const kind = (type: string) => ({ shadow: { type: 'mlab_type', fields: { TYPE: type } } });
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
      block('mlab_exact_damage', { inputs: { AMOUNT: n(40) } }),
      block('mlab_level_damage'),
      block('mlab_ohko'),
      block('mlab_multi_hit'),
      block('mlab_as_type', { inputs: { TYPE: kind('Fire') } }),
      block('mlab_hurt', { fields: { WHO: 'user' }, inputs: { AMOUNT: n(50) } }),
      block('mlab_recoil_max_hp'),
      block('mlab_faint_user'),
    ],
  },
  {
    id: 'heal',
    name: 'Healing',
    color: COLORS.heal,
    blocks: [block('mlab_heal'), block('mlab_restore', { fields: { WHO: 'target' }, inputs: { AMOUNT: n(50) } })],
  },
  {
    id: 'status',
    name: 'Status',
    color: COLORS.status,
    blocks: [
      block('mlab_status', { fields: { CHANCE: 30 } }),
      block('mlab_cure', { fields: { WHO: 'target' } }),
      block('mlab_confuse_target'),
      block('mlab_confuse_self'),
      block('mlab_flinch'),
      block('mlab_bind'),
      block('mlab_free'),
      block('mlab_protect'),
      block('mlab_break_protect'),
    ],
  },
  {
    id: 'stats',
    name: 'Stats',
    color: COLORS.stats,
    blocks: [
      block('mlab_stats'),
      block('mlab_reset_stats', { fields: { WHO: 'target' } }),
      block('mlab_steal_boosts'),
      block('mlab_boost_next', { fields: { TYPE: 'Electric' } }),
      block('mlab_stage'),
      block('mlab_raised_stages'),
    ],
  },
  { id: 'field', name: 'Field', color: COLORS.field, blocks: [block('mlab_weather'), block('mlab_clear_weather')] },
  {
    id: 'effects',
    name: 'Effects',
    color: COLORS.effects,
    blocks: [
      block('mlab_effect', { fields: { SCOPE: 'user_side' } }),
      block('mlab_rule_damage_taken'),
      block('mlab_rule_damage_dealt'),
      block('mlab_rule_damage'),
      block('mlab_rule_heal'),
      block('mlab_rule_drain', { fields: { TO: 'user' } }),
      block('mlab_rule_stat', { fields: { STAT: 'Speed' } }),
      block('mlab_rule_block_status'),
      block('mlab_rule_block_drops'),
      block('mlab_rule_effect_chance'),
      block('mlab_rule_move_type', { fields: { FROM: 'Normal', TO: 'Electric' } }),
      block('mlab_rule_grounded'),
      block('mlab_rule_without_type', { fields: { TYPE: 'Flying' } }),
      block('mlab_rule_endure'),
      block('mlab_rule_always_hit'),
      block('mlab_rule_trapped'),
      block('mlab_end_effect', { fields: { SCOPE: 'target_side' } }),
      block('mlab_end_group', { fields: { SCOPE: 'field' } }),
      block('mlab_effect_active', { fields: { SCOPE: 'field' } }),
      block('mlab_mark', { inputs: { VALUE: n(1) } }),
      block('mlab_unmark'),
      block('mlab_has_mark', { fields: { SCOPE: 'target' } }),
      block('mlab_mark_value'),
    ],
  },
  {
    id: 'turns',
    name: 'Turns',
    color: COLORS.turns,
    blocks: [
      block('mlab_force_move', { inputs: { TURNS: n(2) } }),
      block('mlab_break_sequence'),
      block('mlab_recharge'),
      block('mlab_hide'),
      block('mlab_unhide'),
      block('mlab_if_reaches'),
      block('mlab_sure'),
      block('mlab_on_turn_start'),
      block('mlab_watch_hits'),
      block('mlab_on_hit'),
      block('mlab_on_interrupt'),
      block('mlab_hurry'),
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
      block('mlab_for_each'),
      block('mlab_chance'),
      block('mlab_compare', { inputs: { A: n(0), B: n(0) } }),
      block('mlab_and_or'),
      block('mlab_not'),
      block('mlab_arith', { inputs: { A: n(10), B: n(2) }, fields: { OP: 'MULTIPLY' } }),
      block('mlab_min_max', { inputs: { A: n(1), B: n(100) } }),
      block('mlab_round', { inputs: { A: n(1) } }),
      block('mlab_random', { inputs: { FROM: n(2), TO: n(3) } }),
    ],
  },
  {
    id: 'info',
    name: 'Battle info',
    color: COLORS.info,
    blocks: [
      block('mlab_status_is', { fields: { WHO: 'target' } }),
      block('mlab_type_is', { fields: { WHO: 'target' } }),
      block('mlab_state_is', { fields: { WHO: 'target' } }),
      block('mlab_pair_is'),
      block('mlab_text_is', { fields: { WHO: 'target' } }),
      block('mlab_weather_is'),
      block('mlab_env_is'),
      block('mlab_hit_landed'),
      block('mlab_hit_by_target'),
      block('mlab_hit_was'),
      block('mlab_first_turn'),
      block('mlab_last_turn'),
      block('mlab_in_sequence'),
      block('mlab_first_turn_out'),
      block('mlab_used_all'),
      block('mlab_last_failed'),
      block('mlab_repeat_works'),
      block('mlab_ally_fainted'),
      block('mlab_hp'),
      block('mlab_turn'),
      block('mlab_last_damage'),
      block('mlab_streak'),
      block('mlab_count'),
      block('mlab_iv'),
      block('mlab_type_by_number', { inputs: { NUMBER: n(0) } }),
      block('mlab_first_type'),
    ],
  },
  {
    id: 'traits',
    name: 'Items and traits',
    color: COLORS.traits,
    blocks: [
      block('mlab_steal_item'),
      block('mlab_take_item', { fields: { WHO: 'target' } }),
      block('mlab_give_item'),
      block('mlab_suppress_ability', { fields: { WHO: 'target' } }),
      block('mlab_set_type', { inputs: { TYPE: kind('Water') } }),
      block('mlab_lose_type', { fields: { TYPE: 'Fire' } }),
      block('mlab_set_weight', { inputs: { WEIGHT: n(50) } }),
      block('mlab_payout', { inputs: { AMOUNT: n(100) } }),
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
  effect_blocks: COLORS.effects,
  trait_blocks: COLORS.traits,
};
