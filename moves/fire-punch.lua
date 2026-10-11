-- Move 7: Fire Punch. Identity and parameters: pokemon_data/assets/moves/7.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75},
  {kind = Effect.Status, status = Status.Burned, chance = 0.1}
}}

move.id = "fire-punch"
move.name = "Fire Punch"
move.type = Type.Fire
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
return move
