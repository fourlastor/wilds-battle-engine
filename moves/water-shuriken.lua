-- Move 594: Water Shuriken. Identity and parameters: pokemon_data/assets/moves/594.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)

  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:multi_hit(15, 2, 5, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "water-shuriken"
move.name = "Water Shuriken"
move.type = Type.Water
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 1
move.target = Target.Selected
move.flags = {}
return move
