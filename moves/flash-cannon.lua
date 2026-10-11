-- Move 430: Flash Cannon. Identity and parameters: pokemon_data/assets/moves/430.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Stats, chance = 0.1, self = false, stages = {[Stat.SpDefense] = -1}}
}}

move.id = "flash-cannon"
move.name = "Flash Cannon"
move.type = Type.Steel
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
