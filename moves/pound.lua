-- Move 1: Pound. Identity and parameters: pokemon_data/assets/moves/1.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "pound"
move.name = "Pound"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 35
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
