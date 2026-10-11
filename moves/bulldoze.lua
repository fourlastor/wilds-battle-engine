-- Move 523: Bulldoze. Identity and parameters: pokemon_data/assets/moves/523.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(60, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:change_stat(pokemon, Stat.Speed, -1)
    end
  end
end}

move.id = "bulldoze"
move.name = "Bulldoze"
move.type = Type.Ground
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {}
return move
