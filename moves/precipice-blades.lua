-- Move 619: Precipice Blades. Identity and parameters: pokemon_data/assets/moves/619.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(120, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "precipice-blades"
move.name = "Precipice Blades"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.85
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
return move
