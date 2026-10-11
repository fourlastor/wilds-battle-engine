-- Move 231: Iron Tail. Identity and parameters: pokemon_data/assets/moves/231.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 100},
  {kind = Effect.Stats, chance = 0.3, self = false, stages = {[Stat.Defense] = -1}}
}}

move.id = "iron-tail"
move.name = "Iron Tail"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.75
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
