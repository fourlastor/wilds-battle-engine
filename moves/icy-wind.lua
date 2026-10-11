-- Move 196: Icy Wind. Identity and parameters: pokemon_data/assets/moves/196.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(55, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:change_stat(pokemon, Stat.Speed, -1)
    end
  end
end}

move.id = "icy-wind"
move.name = "Icy Wind"
move.type = Type.Ice
move.category = Category.Special
move.pp = 15
move.accuracy = 0.95
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"wind"}
return move
