-- Move 458: Double Hit. Identity and parameters: pokemon_data/assets/moves/458.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:multi_hit(35, 2, 2, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
end}

move.id = "double-hit"
move.name = "Double Hit"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
