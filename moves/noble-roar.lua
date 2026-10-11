-- Move 568: Noble Roar. Identity and parameters: pokemon_data/assets/moves/568.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = -1, [Stat.SpAttack] = -1}}
}}

move.id = "noble-roar"
move.name = "Noble Roar"
move.type = Type.Normal
move.category = Category.Status
move.pp = 30
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "sound"}
return move
