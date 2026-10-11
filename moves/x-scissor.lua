-- Move 404: X-Scissor. Identity and parameters: pokemon_data/assets/moves/404.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80}
}}

move.id = "x-scissor"
move.name = "X-Scissor"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "slicing"}
return move
