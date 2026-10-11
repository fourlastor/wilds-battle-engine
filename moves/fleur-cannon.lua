-- Move 705: Fleur Cannon. Identity and parameters: pokemon_data/assets/moves/705.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 130},
  {kind = Effect.Stats, chance = 1, self = true, stages = {[Stat.SpAttack] = -2}}
}}

move.id = "fleur-cannon"
move.name = "Fleur Cannon"
move.type = Type.Fairy
move.category = Category.Special
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
