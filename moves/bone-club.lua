-- Move 125: Bone Club. Identity and parameters: pokemon_data/assets/moves/125.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 65},
  {kind = Effect.Flinch, chance = 0.1}
}}

move.id = "bone-club"
move.name = "Bone Club"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 20
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"sheer_force"}
return move
