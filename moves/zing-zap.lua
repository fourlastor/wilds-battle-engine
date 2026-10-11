-- Move 716: Zing Zap. Identity and parameters: pokemon_data/assets/moves/716.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Flinch, chance = 0.3}
}}

move.id = "zing-zap"
move.name = "Zing Zap"
move.type = Type.Electric
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
