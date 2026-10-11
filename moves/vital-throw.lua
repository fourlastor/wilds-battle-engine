-- Move 233: Vital Throw. Identity and parameters: pokemon_data/assets/moves/233.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 70}
}}

move.id = "vital-throw"
move.name = "Vital Throw"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = -1
move.target = Target.Selected
move.flags = {"contact"}
return move
