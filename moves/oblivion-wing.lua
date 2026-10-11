-- Move 613: Oblivion Wing. Identity and parameters: pokemon_data/assets/moves/613.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80, drain = 0.75}
}}

move.id = "oblivion-wing"
move.name = "Oblivion Wing"
move.type = Type.Flying
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"heal"}
return move
