-- Move 36: Take Down. Identity and parameters: pokemon_data/assets/moves/36.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90, recoil = 0.25}
}}

move.id = "take-down"
move.name = "Take Down"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 20
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
