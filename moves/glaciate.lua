-- Move 549: Glaciate. Identity and parameters: pokemon_data/assets/moves/549.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(65, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:change_stat(pokemon, Stat.Speed, -1)
    end
  end
end}

move.id = "glaciate"
move.name = "Glaciate"
move.type = Type.Ice
move.category = Category.Special
move.pp = 10
move.accuracy = 0.95
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
move.base_power = 65
return move
