-- Move 88: Rock Throw. Identity and parameters: pokemon_data/assets/moves/88.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 50}
}}

move.id = "rock-throw"
move.name = "Rock Throw"
move.type = Type.Rock
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
