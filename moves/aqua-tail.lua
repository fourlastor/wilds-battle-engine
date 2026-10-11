-- Move 401: Aqua Tail. Identity and parameters: pokemon_data/assets/moves/401.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90}
}}

move.id = "aqua-tail"
move.name = "Aqua Tail"
move.type = Type.Water
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
