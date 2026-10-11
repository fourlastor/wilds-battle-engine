-- Move 670: Leafage. Identity and parameters: pokemon_data/assets/moves/670.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "leafage"
move.name = "Leafage"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 40
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
