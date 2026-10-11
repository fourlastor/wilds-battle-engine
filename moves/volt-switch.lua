-- Move 521: Volt Switch. Identity and parameters: pokemon_data/assets/moves/521.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Switching, bench selection and switch interception are not implemented; only the on-field damage/stat effect runs.

local move = {effects = {
  {kind = Effect.Damage, power = 70}
}}

move.id = "volt-switch"
move.name = "Volt Switch"
move.type = Type.Electric
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
