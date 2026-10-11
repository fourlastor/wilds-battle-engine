-- Move 665: Ice Hammer. Identity and parameters: pokemon_data/assets/moves/665.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100},
  {kind = Effect.Stats, chance = 1, self = true, stages = {[Stat.Speed] = -1}}
}}

move.id = "ice-hammer"
move.name = "Ice Hammer"
move.type = Type.Ice
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
