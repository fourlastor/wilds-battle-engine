-- Move 680: Fire Lash. Identity and parameters: pokemon_data/assets/moves/680.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Defense] = -1}}
}}

move.id = "fire-lash"
move.name = "Fire Lash"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
