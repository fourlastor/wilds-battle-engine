-- Move 211: Steel Wing. Identity and parameters: pokemon_data/assets/moves/211.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 70},
  {kind = Effect.Stats, chance = 0.1, self = true, stages = {[Stat.Defense] = 1}}
}}

move.id = "steel-wing"
move.name = "Steel Wing"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 25
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
