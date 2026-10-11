-- Move 86: Thunder Wave. Identity and parameters: pokemon_data/assets/moves/86.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Status, status = Status.Paralyzed, chance = 1}
}}

move.id = "thunder-wave"
move.name = "Thunder Wave"
move.type = Type.Electric
move.category = Category.Status
move.pp = 20
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
