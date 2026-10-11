-- Move 552: Fiery Dance. Identity and parameters: pokemon_data/assets/moves/552.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Stats, chance = 0.5, self = true, stages = {[Stat.SpAttack] = 1}}
}}

move.id = "fiery-dance"
move.name = "Fiery Dance"
move.type = Type.Fire
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"dance", "sheer_force"}
return move
