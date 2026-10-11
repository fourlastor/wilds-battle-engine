-- Move 543: Head Charge. Identity and parameters: pokemon_data/assets/moves/543.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120, recoil = 0.25}
}}

move.id = "head-charge"
move.name = "Head Charge"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
