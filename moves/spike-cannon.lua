-- Move 131: Spike Cannon. Identity and parameters: pokemon_data/assets/moves/131.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)

  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:multi_hit(20, 2, 5, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "spike-cannon"
move.name = "Spike Cannon"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
