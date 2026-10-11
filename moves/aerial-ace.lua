-- Move 332: Aerial Ace. Identity and parameters: pokemon_data/assets/moves/332.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60}
}}

move.id = "aerial-ace"
move.name = "Aerial Ace"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "slicing"}
return move
