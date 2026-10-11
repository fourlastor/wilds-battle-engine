-- Move 34: Body Slam. Identity and parameters: pokemon_data/assets/moves/34.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 85},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 0.3}
}}

move.id = "body-slam"
move.name = "Body Slam"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
