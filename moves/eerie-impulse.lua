-- Move 598: Eerie Impulse. Identity and parameters: pokemon_data/assets/moves/598.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.SpAttack] = -2}}
}}

move.id = "eerie-impulse"
move.name = "Eerie Impulse"
move.type = Type.Electric
move.category = Category.Status
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
