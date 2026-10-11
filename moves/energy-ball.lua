-- Move 412: Energy Ball. Identity and parameters: pokemon_data/assets/moves/412.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90},
  {kind = Effect.Stats, chance = 0.1, self = false, stages = {[Stat.SpDefense] = -1}}
}}

move.id = "energy-ball"
move.name = "Energy Ball"
move.type = Type.Grass
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet", "sheer_force"}
return move
