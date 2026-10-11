-- Move 451: Charge Beam. Identity and parameters: pokemon_data/assets/moves/451.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 50},
  {kind = Effect.Stats, chance = 0.7, self = true, stages = {[Stat.SpAttack] = 1}}
}}

move.id = "charge-beam"
move.name = "Charge Beam"
move.type = Type.Electric
move.category = Category.Special
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
