-- Move 238: Cross Chop. Identity and parameters: pokemon_data/assets/moves/238.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100, high_crit = true}
}}

move.id = "cross-chop"
move.name = "Cross Chop"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.8
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
