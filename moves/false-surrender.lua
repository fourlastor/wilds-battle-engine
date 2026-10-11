-- Move 793: False Surrender. Identity and parameters: pokemon_data/assets/moves/793.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 80}
}}

move.id = "false-surrender"
move.name = "False Surrender"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
