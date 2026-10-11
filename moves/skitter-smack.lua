-- Move 806: Skitter Smack. Identity and parameters: pokemon_data/assets/moves/806.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 70},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpAttack] = -1}}
}}

move.id = "skitter-smack"
move.name = "Skitter Smack"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
