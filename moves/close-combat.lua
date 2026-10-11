-- Move 370: Close Combat. Identity and parameters: pokemon_data/assets/moves/370.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120},
  {kind = Effect.Stats, chance = 1, self = true, stages = {[Stat.Defense] = -1, [Stat.SpDefense] = -1}}
}}

move.id = "close-combat"
move.name = "Close Combat"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
