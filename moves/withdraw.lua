-- Move 110: Withdraw. Identity and parameters: pokemon_data/assets/moves/110.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Defense] = 1}}
}}

move.id = "withdraw"
move.name = "Withdraw"
move.type = Type.Water
move.category = Category.Status
move.pp = 40
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
