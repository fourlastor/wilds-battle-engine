-- Move 224: Megahorn. Identity and parameters: pokemon_data/assets/moves/224.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 120}
}}

move.id = "megahorn"
move.name = "Megahorn"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
