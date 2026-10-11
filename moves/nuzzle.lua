-- Move 609: Nuzzle. Identity and parameters: pokemon_data/assets/moves/609.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 20},
  {kind = Effect.Status, status = Status.Paralyzed, chance = 1}
}}

move.id = "nuzzle"
move.name = "Nuzzle"
move.type = Type.Electric
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
