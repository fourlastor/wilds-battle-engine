-- Move 44: Bite. Identity and parameters: pokemon_data/assets/moves/44.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60},
  {kind = Effect.Flinch, chance = 0.3}
}}

move.id = "bite"
move.name = "Bite"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "bite"}
return move
