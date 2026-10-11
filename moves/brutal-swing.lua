-- Move 693: Brutal Swing. Identity and parameters: pokemon_data/assets/moves/693.json.
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

move.id = "brutal-swing"
move.name = "Brutal Swing"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {"contact"}
move.base_power = 60
return move
