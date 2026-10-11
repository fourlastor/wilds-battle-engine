-- Move 443: Magnet Bomb. Identity and parameters: pokemon_data/assets/moves/443.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60}
}}

move.id = "magnet-bomb"
move.name = "Magnet Bomb"
move.type = Type.Steel
move.category = Category.Physical
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
return move
