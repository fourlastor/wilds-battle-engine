-- Move 129: Swift. Identity and parameters: pokemon_data/assets/moves/129.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(60, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "swift"
move.name = "Swift"
move.type = Type.Normal
move.category = Category.Special
move.pp = 20
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
return move
