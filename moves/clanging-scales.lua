-- Move 691: Clanging Scales. Identity and parameters: pokemon_data/assets/moves/691.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(110, {target = pokemon})
    if hit.hit then
      connected = true
    end
  end
  if connected then
    ctx:change_stat(ctx.user, Stat.Defense, -1)
  end
end}

move.id = "clanging-scales"
move.name = "Clanging Scales"
move.type = Type.Dragon
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"sound"}
move.base_power = 110
return move
