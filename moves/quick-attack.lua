-- Move 98: Quick Attack. Identity and parameters: pokemon_data/assets/moves/98.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "quick-attack"
move.name = "Quick Attack"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 30
move.accuracy = 1
move.priority = 1
move.target = Target.Selected
move.flags = {"contact"}
return move
