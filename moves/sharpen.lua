-- Move 159: Sharpen. Identity and parameters: pokemon_data/assets/moves/159.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = 1}}
}}

move.id = "sharpen"
move.name = "Sharpen"
move.type = Type.Normal
move.category = Category.Status
move.pp = 30
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
