-- Move 369: U-turn. Identity and parameters: pokemon_data/assets/moves/369.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Switching, bench selection and switch interception are not implemented; only the on-field damage/stat effect runs.

local move = {effects = {
  {kind = Effect.Damage, power = 70}
}}

move.id = "u-turn"
move.name = "U-turn"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
