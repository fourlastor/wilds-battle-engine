-- Move 177: Aeroblast. Identity and parameters: pokemon_data/assets/moves/177.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100, high_crit = true}
}}

move.id = "aeroblast"
move.name = "Aeroblast"
move.type = Type.Flying
move.category = Category.Special
move.pp = 5
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"wind"}
return move
