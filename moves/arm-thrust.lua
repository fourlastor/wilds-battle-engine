-- Move 292: Arm Thrust. Identity and parameters: pokemon_data/assets/moves/292.json.
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

move.id = "arm-thrust"
move.name = "Arm Thrust"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
