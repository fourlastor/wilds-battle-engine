-- Move 97: Agility. Identity and parameters: pokemon_data/assets/moves/97.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Speed] = 2}}
}}

move.id = "agility"
move.name = "Agility"
move.type = Type.Psychic
move.category = Category.Status
move.pp = 30
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
