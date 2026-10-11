-- Move 133: Amnesia. Identity and parameters: pokemon_data/assets/moves/133.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpDefense] = 2}}
}}

move.id = "amnesia"
move.name = "Amnesia"
move.type = Type.Psychic
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
