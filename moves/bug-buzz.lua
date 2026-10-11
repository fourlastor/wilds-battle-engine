-- Move 405: Bug Buzz. Identity and parameters: pokemon_data/assets/moves/405.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90},
  {kind = Effect.Stats, chance = 0.1, self = false, stages = {[Stat.SpDefense] = -1}}
}}

move.id = "bug-buzz"
move.name = "Bug Buzz"
move.type = Type.Bug
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sound"}
return move
