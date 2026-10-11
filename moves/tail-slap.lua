-- Move 541: Tail Slap. Identity and parameters: pokemon_data/assets/moves/541.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)

  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:multi_hit(25, 2, 5, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "tail-slap"
move.name = "Tail Slap"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 25
return move
