-- Move 17: Wing Attack. Identity and parameters: pokemon_data/assets/moves/17.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60}
}}

move.id = "wing-attack"
move.name = "Wing Attack"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 35
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
