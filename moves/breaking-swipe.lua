-- Move 784: Breaking Swipe. Identity and parameters: pokemon_data/assets/moves/784.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(60, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:change_stat(pokemon, Stat.Attack, -1)
    end
  end
end}

move.id = "breaking-swipe"
move.name = "Breaking Swipe"
move.type = Type.Dragon
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"contact"}
return move
