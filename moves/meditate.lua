-- Move 96: Meditate. Identity and parameters: pokemon_data/assets/moves/96.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = 1}}
}}

move.id = "meditate"
move.name = "Meditate"
move.type = Type.Psychic
move.category = Category.Status
move.pp = 40
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
