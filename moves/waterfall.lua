-- Move 127: Waterfall. Identity and parameters: pokemon_data/assets/moves/127.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Flinch, chance = 0.2}
}}

move.id = "waterfall"
move.name = "Waterfall"
move.type = Type.Water
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
