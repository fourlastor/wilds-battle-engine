-- Move 410: Vacuum Wave. Identity and parameters: pokemon_data/assets/moves/410.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "vacuum-wave"
move.name = "Vacuum Wave"
move.type = Type.Fighting
move.category = Category.Special
move.pp = 30
move.accuracy = 1
move.priority = 1
move.target = Target.Selected
move.flags = {}
return move
