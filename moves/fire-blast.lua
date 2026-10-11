-- Move 126: Fire Blast. Identity and parameters: pokemon_data/assets/moves/126.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 110},
  {kind = Effect.Status, status = Status.Burned, chance = 0.1}
}}

move.id = "fire-blast"
move.name = "Fire Blast"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
