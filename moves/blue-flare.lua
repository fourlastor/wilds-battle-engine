-- Move 551: Blue Flare. Identity and parameters: pokemon_data/assets/moves/551.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 130},
  {kind = Effect.Status, status = Status.Burned, chance = 0.2}
}}

move.id = "blue-flare"
move.name = "Blue Flare"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
