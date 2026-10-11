-- Move 158: Hyper Fang. Identity and parameters: pokemon_data/assets/moves/158.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80},
  {kind = Effect.Flinch, chance = 0.1}
}}

move.id = "hyper-fang"
move.name = "Hyper Fang"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "bite", "sheer_force"}
return move
