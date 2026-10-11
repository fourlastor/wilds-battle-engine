-- Move 457: Head Smash. Identity and parameters: pokemon_data/assets/moves/457.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 150, recoil = 0.5}
}}

move.id = "head-smash"
move.name = "Head Smash"
move.type = Type.Rock
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.8
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
