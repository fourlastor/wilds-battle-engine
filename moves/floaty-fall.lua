-- Move 731: Floaty Fall. Identity and parameters: pokemon_data/assets/moves/731.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90},
  {kind = Effect.Flinch, chance = 0.3}
}}

move.id = "floaty-fall"
move.name = "Floaty Fall"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 15
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
