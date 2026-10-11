-- Move 342: Poison Tail. Identity and parameters: pokemon_data/assets/moves/342.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 50, high_crit = true},
  {kind = Effect.Status, status = Status.Poisoned, chance = 0.1}
}}

move.id = "poison-tail"
move.name = "Poison Tail"
move.type = Type.Poison
move.category = Category.Physical
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
