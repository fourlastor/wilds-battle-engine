-- Move 575: Parting Shot. Identity and parameters: pokemon_data/assets/moves/575.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Switching, bench selection and switch interception are not implemented; only the on-field damage/stat effect runs.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = -1, [Stat.SpAttack] = -1}}
}}

move.id = "parting-shot"
move.name = "Parting Shot"
move.type = Type.Dark
move.category = Category.Status
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "sound", "incomplete"}
return move
