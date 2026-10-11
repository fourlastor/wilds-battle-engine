-- Move 701: Pulverizing Pancake. Identity and parameters: pokemon_data/assets/moves/701.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.

local move = {effects = {
  {kind = Effect.Damage, power = 210}
}}

move.id = "pulverizing-pancake"
move.name = "Pulverizing Pancake"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
