-- Move 296: Mist Ball. Identity and parameters: pokemon_data/assets/moves/296.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 95},
  {kind = Effect.Stats, chance = 0.5, self = false, stages = {[Stat.SpAttack] = -1}}
}}

move.id = "mist-ball"
move.name = "Mist Ball"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet", "sheer_force"}
return move
