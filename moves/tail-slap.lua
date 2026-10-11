-- Move 541: Tail Slap. Identity and parameters: pokemon_data/assets/moves/541.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local roll = ctx:random_int(1, 100)
  local count = roll <= 35 and 2 or roll <= 70 and 3 or roll <= 85 and 4 or 5
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:multi_hit(25, count, count, {target = pokemon})
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
return move
