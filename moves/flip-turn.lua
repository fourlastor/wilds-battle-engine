-- Move 812: Flip Turn. Identity and parameters: pokemon_data/assets/moves/812.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Switching, bench selection and switch interception are not implemented; only the on-field damage/stat effect runs.

local move = {effects = {
  {kind = Effect.Damage, power = 60}
}}

move.id = "flip-turn"
move.name = "Flip Turn"
move.type = Type.Water
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
