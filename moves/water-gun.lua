-- Move 55: Water Gun. Identity and parameters: pokemon_data/assets/moves/55.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "water-gun"
move.name = "Water Gun"
move.type = Type.Water
move.category = Category.Special
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
