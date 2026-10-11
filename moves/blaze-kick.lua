-- Move 299: Blaze Kick. Identity and parameters: pokemon_data/assets/moves/299.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 85, high_crit = true},
  {kind = Effect.Status, status = Status.Burned, chance = 0.1}
}}

move.id = "blaze-kick"
move.name = "Blaze Kick"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
