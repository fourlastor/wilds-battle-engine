-- Move 142: Lovely Kiss. Identity and parameters: pokemon_data/assets/moves/142.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Status, status = Status.Asleep, chance = 1}
}}

move.id = "lovely-kiss"
move.name = "Lovely Kiss"
move.type = Type.Normal
move.category = Category.Status
move.pp = 10
move.accuracy = 0.75
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
