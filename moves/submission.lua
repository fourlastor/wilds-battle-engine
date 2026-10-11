-- Move 66: Submission. Identity and parameters: pokemon_data/assets/moves/66.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80, recoil = 0.25}
}}

move.id = "submission"
move.name = "Submission"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 20
move.accuracy = 0.8
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
