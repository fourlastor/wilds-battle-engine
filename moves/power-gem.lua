-- Move 408: Power Gem. Identity and parameters: pokemon_data/assets/moves/408.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80}
}}

move.id = "power-gem"
move.name = "Power Gem"
move.type = Type.Rock
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
