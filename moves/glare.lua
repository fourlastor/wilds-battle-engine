-- Move 137: Glare. Identity and parameters: pokemon_data/assets/moves/137.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Status, status = Status.Paralyzed, chance = 1}
}}

move.id = "glare"
move.name = "Glare"
move.type = Type.Normal
move.category = Category.Status
move.pp = 30
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
