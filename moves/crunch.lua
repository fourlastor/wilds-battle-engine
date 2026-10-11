-- Move 242: Crunch. Identity and parameters: pokemon_data/assets/moves/242.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Stats, chance = 0.2, self = false, stages = {[Stat.Defense] = -1}}
}}

move.id = "crunch"
move.name = "Crunch"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "bite", "sheer_force"}
return move
