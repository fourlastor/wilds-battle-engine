-- Move 491: Acid Spray. Identity and parameters: pokemon_data/assets/moves/491.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpDefense] = -2}}
}}

move.id = "acid-spray"
move.name = "Acid Spray"
move.type = Type.Poison
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
return move
