-- Move 700: Stoked Sparksurfer. Identity and parameters: pokemon_data/assets/moves/700.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.

local move = {effects = {
  {kind = Effect.Damage, power = 175},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 1}
}}

move.id = "stoked-sparksurfer"
move.name = "Stoked Sparksurfer"
move.type = Type.Electric
move.category = Category.Special
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
