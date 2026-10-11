-- Move 61: Bubble Beam. Identity and parameters: pokemon_data/assets/moves/61.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Stats, chance = 0.1, self = false, stages = {[Stat.Speed] = -1}}
}}

move.id = "bubble-beam"
move.name = "Bubble Beam"
move.type = Type.Water
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
