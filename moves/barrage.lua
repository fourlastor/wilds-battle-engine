-- Move 140: Barrage. Identity and parameters: pokemon_data/assets/moves/140.json.
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

move.id = "barrage"
move.name = "Barrage"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 20
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
move.base_power = 15
return move
