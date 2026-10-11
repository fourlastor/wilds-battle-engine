-- Move 155: Bonemerang. Identity and parameters: pokemon_data/assets/moves/155.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:multi_hit(50, 2, 2, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "bonemerang"
move.name = "Bonemerang"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
