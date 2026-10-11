-- Move 440: Cross Poison. Identity and parameters: pokemon_data/assets/moves/440.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 70, high_crit = true},
  {kind = Effect.Status, status = Status.Poisoned, chance = 0.1}
}}

move.id = "cross-poison"
move.name = "Cross Poison"
move.type = Type.Poison
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "slicing"}
return move
