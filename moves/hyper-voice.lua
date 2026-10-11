-- Move 304: Hyper Voice. Identity and parameters: pokemon_data/assets/moves/304.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(90, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "hyper-voice"
move.name = "Hyper Voice"
move.type = Type.Normal
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"sound"}
return move
