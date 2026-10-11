-- Move 528: Wild Charge. Identity and parameters: pokemon_data/assets/moves/528.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90, recoil = 0.25}
}}

move.id = "wild-charge"
move.name = "Wild Charge"
move.type = Type.Electric
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
