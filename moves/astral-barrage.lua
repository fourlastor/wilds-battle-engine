-- Move 825: Astral Barrage. Identity and parameters: pokemon_data/assets/moves/825.json.
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

move.id = "astral-barrage"
move.name = "Astral Barrage"
move.type = Type.Ghost
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
return move
