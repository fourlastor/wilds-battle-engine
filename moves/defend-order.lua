-- Move 455: Defend Order. Identity and parameters: pokemon_data/assets/moves/455.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Defense] = 1, [Stat.SpDefense] = 1}}
}}

move.id = "defend-order"
move.name = "Defend Order"
move.type = Type.Bug
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch"}
return move
