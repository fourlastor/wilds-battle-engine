-- Move 345: Magical Leaf. Identity and parameters: pokemon_data/assets/moves/345.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 60}
}}

move.id = "magical-leaf"
move.name = "Magical Leaf"
move.type = Type.Grass
move.category = Category.Special
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
