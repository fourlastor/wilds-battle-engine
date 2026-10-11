-- Move 556: Icicle Crash. Identity and parameters: pokemon_data/assets/moves/556.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 85},
  {kind = Effect.Flinch, chance = 0.3}
}}

move.id = "icicle-crash"
move.name = "Icicle Crash"
move.type = Type.Ice
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
