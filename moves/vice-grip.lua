-- Move 11: Vise Grip. Identity and parameters: pokemon_data/assets/moves/11.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 55}
}}

move.id = "vice-grip"
move.name = "Vise Grip"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 30
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
