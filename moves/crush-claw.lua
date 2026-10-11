-- Move 306: Crush Claw. Identity and parameters: pokemon_data/assets/moves/306.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75},
  {kind = Effect.Stats, chance = 0.5, self = false, stages = {[Stat.Defense] = -1}}
}}

move.id = "crush-claw"
move.name = "Crush Claw"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
