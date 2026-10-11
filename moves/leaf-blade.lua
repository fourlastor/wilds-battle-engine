-- Move 348: Leaf Blade. Identity and parameters: pokemon_data/assets/moves/348.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {effects = {
  {kind = Effect.Damage, power = 90, high_crit = true}
}}

move.id = "leaf-blade"
move.name = "Leaf Blade"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "slicing"}
return move
