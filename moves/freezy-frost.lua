-- Move 739: Freezy Frost. Identity and parameters: pokemon_data/assets/moves/739.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
if ctx:damage(100).hit then
  for _, pokemon in ipairs(ctx.everyone) do ctx:reset_stats(pokemon) end
end
end}

move.id = "freezy-frost"
move.name = "Freezy Frost"
move.type = Type.Ice
move.category = Category.Special
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
