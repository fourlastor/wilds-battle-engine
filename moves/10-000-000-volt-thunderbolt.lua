-- Move 719: 10,000,000 Volt Thunderbolt. Identity and parameters: pokemon_data/assets/moves/719.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.
-- INCOMPLETE: The damage API supports one increased critical-hit stage, not this move's higher critical-hit ratio.

local move = {effects = {
  {kind = Effect.Damage, power = 195, high_crit = true}
}}

move.id = "10-000-000-volt-thunderbolt"
move.name = "10,000,000 Volt Thunderbolt"
move.type = Type.Electric
move.category = Category.Special
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
