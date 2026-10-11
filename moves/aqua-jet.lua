-- Move 453: Aqua Jet. Identity and parameters: pokemon_data/assets/moves/453.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "aqua-jet"
move.name = "Aqua Jet"
move.type = Type.Water
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 1
move.target = Target.Selected
move.flags = {"contact"}
return move
