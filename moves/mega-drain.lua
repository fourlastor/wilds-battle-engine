-- Move 72: Mega Drain. Identity and parameters: pokemon_data/assets/moves/72.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40, drain = 0.5}
}}

move.id = "mega-drain"
move.name = "Mega Drain"
move.type = Type.Grass
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
