-- Move 490: Low Sweep. Identity and parameters: pokemon_data/assets/moves/490.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Speed] = -1}}
}}

move.id = "low-sweep"
move.name = "Low Sweep"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
