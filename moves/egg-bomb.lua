-- Move 121: Egg Bomb. Identity and parameters: pokemon_data/assets/moves/121.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100}
}}

move.id = "egg-bomb"
move.name = "Egg Bomb"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.75
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
return move
