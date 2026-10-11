-- Move 228: Pursuit. Identity and parameters: pokemon_data/assets/moves/228.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Switching, bench selection and switch interception are not implemented; only the on-field damage/stat effect runs.

local move = {flags = {"contact"}, effects = {{kind = Effect.Damage, power = 40}}}

move.id = "pursuit"
move.name = "Pursuit"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
