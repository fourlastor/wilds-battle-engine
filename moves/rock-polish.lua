-- Move 397: Rock Polish. Identity and parameters: pokemon_data/assets/moves/397.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Speed] = 2}}
}}

move.id = "rock-polish"
move.name = "Rock Polish"
move.type = Type.Rock
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
