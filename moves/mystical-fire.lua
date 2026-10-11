-- Move 595: Mystical Fire. Identity and parameters: pokemon_data/assets/moves/595.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpAttack] = -1}}
}}

move.id = "mystical-fire"
move.name = "Mystical Fire"
move.type = Type.Fire
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
