-- Move 803: Grassy Glide. Identity and parameters: pokemon_data/assets/moves/803.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  priority_terrain = "grassy_terrain",
  script = function(ctx) ctx:damage(55) end,
}

move.id = "grassy-glide"
move.name = "Grassy Glide"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
