-- Move 22: Vine Whip. Identity and parameters: pokemon_data/assets/moves/22.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 45}
}}

move.id = "vine-whip"
move.name = "Vine Whip"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
