-- Move 95: Hypnosis. Identity and parameters: pokemon_data/assets/moves/95.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Status, status = Status.Asleep, chance = 1}
}}

move.id = "hypnosis"
move.name = "Hypnosis"
move.type = Type.Psychic
move.category = Category.Status
move.pp = 20
move.accuracy = 0.6
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
