-- Move 202: Giga Drain. Identity and parameters: pokemon_data/assets/moves/202.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75, drain = 0.5}
}}

move.id = "giga-drain"
move.name = "Giga Drain"
move.type = Type.Grass
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
