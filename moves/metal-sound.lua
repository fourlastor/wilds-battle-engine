-- Move 319: Metal Sound. Identity and parameters: pokemon_data/assets/moves/319.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpDefense] = -2}}
}}

move.id = "metal-sound"
move.name = "Metal Sound"
move.type = Type.Steel
move.category = Category.Status
move.pp = 40
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "sound"}
return move
