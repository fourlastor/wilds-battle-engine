-- Move 398: Poison Jab. Identity and parameters: pokemon_data/assets/moves/398.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Status, status = Status.Poisoned, chance = 0.3}
}}

move.id = "poison-jab"
move.name = "Poison Jab"
move.type = Type.Poison
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
