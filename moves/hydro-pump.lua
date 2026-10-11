-- Move 56: Hydro Pump. Identity and parameters: pokemon_data/assets/moves/56.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 110}
}}

move.id = "hydro-pump"
move.name = "Hydro Pump"
move.type = Type.Water
move.category = Category.Special
move.pp = 5
move.accuracy = 0.8
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
