-- Move 710: Liquidation. Identity and parameters: pokemon_data/assets/moves/710.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 85},
  {kind = Effect.Stats, chance = 0.2, self = false, stages = {[Stat.Defense] = -1}}
}}

move.id = "liquidation"
move.name = "Liquidation"
move.type = Type.Water
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
