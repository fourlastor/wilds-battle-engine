-- Move 442: Iron Head. Identity and parameters: pokemon_data/assets/moves/442.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Flinch, chance = 0.3}
}}

move.id = "iron-head"
move.name = "Iron Head"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
