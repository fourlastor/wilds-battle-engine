-- Move 702: Extreme Evoboost. Identity and parameters: pokemon_data/assets/moves/702.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.

local move = {effects = {
  {kind = Effect.Stats, chance = 1, self = false, stages = {[Stat.Attack] = 2, [Stat.Defense] = 2, [Stat.SpAttack] = 2, [Stat.SpDefense] = 2, [Stat.Speed] = 2}}
}}

move.id = "extreme-evoboost"
move.name = "Extreme Evoboost"
move.type = Type.Normal
move.category = Category.Status
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"incomplete"}
return move
