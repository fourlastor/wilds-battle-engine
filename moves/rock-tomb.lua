-- Move 317: Rock Tomb. Identity and parameters: pokemon_data/assets/moves/317.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Speed] = -1}}
}}

move.id = "rock-tomb"
move.name = "Rock Tomb"
move.type = Type.Rock
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
