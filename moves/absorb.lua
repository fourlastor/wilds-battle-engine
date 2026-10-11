-- Move 71: Absorb. Identity and parameters: pokemon_data/assets/moves/71.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 20, drain = 0.5}
}}

move.id = "absorb"
move.name = "Absorb"
move.type = Type.Grass
move.category = Category.Special
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"heal"}
return move
