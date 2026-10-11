-- Move 574: Disarming Voice. Identity and parameters: pokemon_data/assets/moves/574.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(40, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "disarming-voice"
move.name = "Disarming Voice"
move.type = Type.Fairy
move.category = Category.Special
move.pp = 15
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"sound"}
return move
