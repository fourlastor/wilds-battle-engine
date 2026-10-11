-- Move 509: Circle Throw. Identity and parameters: pokemon_data/assets/moves/509.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Switching, bench selection and switch interception are not implemented; only the on-field damage/stat effect runs.

local move = {effects = {
  {kind = Effect.Damage, power = 60}
}}

move.id = "circle-throw"
move.name = "Circle Throw"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = -6
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
