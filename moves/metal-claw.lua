-- Move 232: Metal Claw. Identity and parameters: pokemon_data/assets/moves/232.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 50},
  {kind = Effect.Stats, chance = 0.1, self = true, stages = {[Stat.Attack] = 1}}
}}

move.id = "metal-claw"
move.name = "Metal Claw"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 35
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
