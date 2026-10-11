-- Move 339: Bulk Up. Identity and parameters: pokemon_data/assets/moves/339.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = 1, [Stat.Defense] = 1}}
}}

move.id = "bulk-up"
move.name = "Bulk Up"
move.type = Type.Fighting
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
