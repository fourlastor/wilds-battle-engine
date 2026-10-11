-- Move 302: Needle Arm. Identity and parameters: pokemon_data/assets/moves/302.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60},
  {kind = Effect.Flinch, chance = 0.3}
}}

move.id = "needle-arm"
move.name = "Needle Arm"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "sheer_force"}
return move
