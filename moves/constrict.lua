-- Move 132: Constrict. Identity and parameters: pokemon_data/assets/moves/132.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 10},
  {kind = Effect.Stats, chance = 0.1, self = false, stages = {[Stat.Speed] = -1}}
}}

move.id = "constrict"
move.name = "Constrict"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 35
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
