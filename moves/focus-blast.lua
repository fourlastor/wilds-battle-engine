-- Move 411: Focus Blast. Identity and parameters: pokemon_data/assets/moves/411.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120},
  {kind = Effect.Stats, chance = 0.1, self = false, stages = {[Stat.SpDefense] = -1}}
}}

move.id = "focus-blast"
move.name = "Focus Blast"
move.type = Type.Fighting
move.category = Category.Special
move.pp = 5
move.accuracy = 0.7
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet", "sheer_force"}
return move
