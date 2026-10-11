-- Move 341: Mud Shot. Identity and parameters: pokemon_data/assets/moves/341.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 55},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Speed] = -1}}
}}

move.id = "mud-shot"
move.name = "Mud Shot"
move.type = Type.Ground
move.category = Category.Special
move.pp = 15
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
