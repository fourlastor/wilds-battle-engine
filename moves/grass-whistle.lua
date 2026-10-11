-- Move 320: Grass Whistle. Identity and parameters: pokemon_data/assets/moves/320.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Status, status = Status.Asleep, chance = 1}
}}

move.id = "grass-whistle"
move.name = "Grass Whistle"
move.type = Type.Grass
move.category = Category.Status
move.pp = 15
move.accuracy = 0.55
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "sound"}
return move
