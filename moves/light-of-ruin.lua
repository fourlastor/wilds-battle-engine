-- Move 617: Light of Ruin. Identity and parameters: pokemon_data/assets/moves/617.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 140, recoil = 0.5}
}}

move.id = "light-of-ruin"
move.name = "Light of Ruin"
move.type = Type.Fairy
move.category = Category.Special
move.pp = 5
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
