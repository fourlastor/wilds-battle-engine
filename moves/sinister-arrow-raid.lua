-- Move 695: Sinister Arrow Raid. Identity and parameters: pokemon_data/assets/moves/695.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.

local move = {effects = {
  {kind = Effect.Damage, power = 180}
}}

move.id = "sinister-arrow-raid"
move.name = "Sinister Arrow Raid"
move.type = Type.Ghost
move.category = Category.Physical
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
