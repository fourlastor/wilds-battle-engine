-- Move 30: Horn Attack. Identity and parameters: pokemon_data/assets/moves/30.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65}
}}

move.id = "horn-attack"
move.name = "Horn Attack"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
