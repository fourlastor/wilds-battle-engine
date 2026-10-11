-- Move 27: Rolling Kick. Identity and parameters: pokemon_data/assets/moves/27.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60},
  {kind = Effect.Flinch, chance = 0.3}
}}

move.id = "rolling-kick"
move.name = "Rolling Kick"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
