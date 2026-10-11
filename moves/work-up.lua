-- Move 526: Work Up. Identity and parameters: pokemon_data/assets/moves/526.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = 1, [Stat.SpAttack] = 1}}
}}

move.id = "work-up"
move.name = "Work Up"
move.type = Type.Normal
move.category = Category.Status
move.pp = 30
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
