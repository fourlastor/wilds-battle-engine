-- Move 466: Ominous Wind. Identity and parameters: pokemon_data/assets/moves/466.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60},
  {kind = Effect.Stats, chance = 0.1, self = true, stages = {[Stat.Attack] = 1, [Stat.Defense] = 1, [Stat.SpAttack] = 1, [Stat.SpDefense] = 1, [Stat.Speed] = 1}}
}}

move.id = "ominous-wind"
move.name = "Ominous Wind"
move.type = Type.Ghost
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
