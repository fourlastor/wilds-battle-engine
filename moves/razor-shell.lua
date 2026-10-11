-- Move 534: Razor Shell. Identity and parameters: pokemon_data/assets/moves/534.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75},
  {kind = Effect.Stats, chance = 0.5, self = false, stages = {[Stat.Defense] = -1}}
}}

move.id = "razor-shell"
move.name = "Razor Shell"
move.type = Type.Water
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "slicing"}
return move
