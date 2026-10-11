-- Move 310: Astonish. Identity and parameters: pokemon_data/assets/moves/310.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 30},
  {kind = Effect.Flinch, chance = 0.3}
}}

move.id = "astonish"
move.name = "Astonish"
move.type = Type.Ghost
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
