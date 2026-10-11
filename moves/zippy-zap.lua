-- Move 729: Zippy Zap. Identity and parameters: pokemon_data/assets/moves/729.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Stats, chance = 1, self = true, stages = {[Stat.Evasion] = 1}}
}}

move.id = "zippy-zap"
move.name = "Zippy Zap"
move.type = Type.Electric
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 2
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
