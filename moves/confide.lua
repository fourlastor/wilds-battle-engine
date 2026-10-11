-- Move 590: Confide. Identity and parameters: pokemon_data/assets/moves/590.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpAttack] = -1}}
}}

move.id = "confide"
move.name = "Confide"
move.type = Type.Normal
move.category = Category.Status
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "sound"}
return move
