-- Move 586: Boomburst. Identity and parameters: pokemon_data/assets/moves/586.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(140, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "boomburst"
move.name = "Boomburst"
move.type = Type.Normal
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {"sound"}
move.base_power = 140
return move
