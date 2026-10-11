-- Move 538: Cotton Guard. Identity and parameters: pokemon_data/assets/moves/538.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Defense] = 3}}
}}

move.id = "cotton-guard"
move.name = "Cotton Guard"
move.type = Type.Grass
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
