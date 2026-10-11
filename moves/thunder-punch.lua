-- Move 9: Thunder Punch. Identity and parameters: pokemon_data/assets/moves/9.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 75},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 0.1}
}}

move.id = "thunder-punch"
move.name = "Thunder Punch"
move.type = Type.Electric
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch", "sheer_force"}
return move
