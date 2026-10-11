-- Move 699: Soul-Stealing 7-Star Strike. Identity and parameters: pokemon_data/assets/moves/699.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.

local move = {effects = {
  {kind = Effect.Damage, power = 195}
}}

move.id = "soul-stealing-7-star-strike"
move.name = "Soul-Stealing 7-Star Strike"
move.type = Type.Ghost
move.category = Category.Physical
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
