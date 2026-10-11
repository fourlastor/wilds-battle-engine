-- Move 618: Origin Pulse. Identity and parameters: pokemon_data/assets/moves/618.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(110, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "origin-pulse"
move.name = "Origin Pulse"
move.type = Type.Water
move.category = Category.Special
move.pp = 10
move.accuracy = 0.85
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"pulse"}
return move
