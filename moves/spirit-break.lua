-- Move 789: Spirit Break. Identity and parameters: pokemon_data/assets/moves/789.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpAttack] = -1}}
}}

move.id = "spirit-break"
move.name = "Spirit Break"
move.type = Type.Fairy
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
