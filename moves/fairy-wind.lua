-- Move 584: Fairy Wind. Identity and parameters: pokemon_data/assets/moves/584.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 40}
}}

move.id = "fairy-wind"
move.name = "Fairy Wind"
move.type = Type.Fairy
move.category = Category.Special
move.pp = 30
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"wind"}
return move
