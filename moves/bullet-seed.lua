-- Move 331: Bullet Seed. Identity and parameters: pokemon_data/assets/moves/331.json.
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

move.id = "bullet-seed"
move.name = "Bullet Seed"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 30
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"bullet"}
return move
