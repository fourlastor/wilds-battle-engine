import * as Blockly from 'blockly/core';
import { COLORS } from './toolbox.ts';

const style = (primary: string, secondary: string, tertiary: string, hat = '') => ({
  colourPrimary: primary,
  colourSecondary: secondary,
  colourTertiary: tertiary,
  hat,
});

export const theme = Blockly.Theme.defineTheme('movelab', {
  name: 'movelab',
  base: Blockly.Themes.Zelos,
  blockStyles: {
    hat_blocks: style(COLORS.hat, '#2A5645', '#0D261C', 'cap'),
    damage_blocks: style(COLORS.damage, '#A82D25', '#96261F'),
    heal_blocks: style(COLORS.heal, '#196A41', '#155B38'),
    status_blocks: style(COLORS.status, '#6238A8', '#563193'),
    stats_blocks: style(COLORS.stats, '#1A5CB0', '#16509B'),
    field_blocks: style(COLORS.field, '#096B74', '#085C64'),
    turn_blocks: style(COLORS.turns, '#9B2969', '#88235C'),
    logic_blocks: style(COLORS.logic, '#8C5000', '#7C4700'),
    info_blocks: style(COLORS.info, '#F2C230', '#D9A800'),
    text_blocks: style(COLORS.text, '#415049', '#38463F'),
    effect_blocks: style(COLORS.effects, '#4D6808', '#425907'),
    trait_blocks: style(COLORS.traits, '#2B435D', '#24394F'),
  },
  componentStyles: {
    workspaceBackgroundColour: '#EDF0EA',
    toolboxBackgroundColour: '#FFFFFF',
    toolboxForegroundColour: '#15231C',
    flyoutBackgroundColour: '#F7F9F6',
    flyoutForegroundColour: '#15231C',
    flyoutOpacity: 1,
    scrollbarColour: '#8D9A90',
    scrollbarOpacity: 0.45,
    insertionMarkerColour: '#15231C',
    insertionMarkerOpacity: 0.25,
  },
  fontStyle: { family: "'Rubik', 'Segoe UI', system-ui, sans-serif", weight: '600', size: 11.5 },
  startHats: false,
});
