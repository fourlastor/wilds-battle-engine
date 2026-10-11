-- Move 247: Shadow Ball. Identity and parameters: pokemon_data/assets/moves/247.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Stats, chance = 0.2, self = false, stages = {[Stat.SpDefense] = -1}}
}}

move.id = "shadow-ball"
move.name = "Shadow Ball"
move.type = Type.Ghost
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
return move
