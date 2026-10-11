-- Move 189: Mud-Slap. Identity and parameters: pokemon_data/assets/moves/189.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 20},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Accuracy] = -1}}
}}

move.id = "mud-slap"
move.name = "Mud-Slap"
move.type = Type.Ground
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
