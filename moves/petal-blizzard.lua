-- Move 572: Petal Blizzard. Identity and parameters: pokemon_data/assets/moves/572.json.
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

move.id = "petal-blizzard"
move.name = "Petal Blizzard"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {"wind"}
return move
