-- Move 585: Moonblast. Identity and parameters: pokemon_data/assets/moves/585.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 95},
  {kind = Effect.Stats, chance = 0.3, self = false, stages = {[Stat.SpAttack] = -1}}
}}

move.id = "moonblast"
move.name = "Moonblast"
move.type = Type.Fairy
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
