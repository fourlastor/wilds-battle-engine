-- Move 145: Bubble. Identity and parameters: pokemon_data/assets/moves/145.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(40, {target = pokemon})
    if hit.hit then
      connected = true
      if ctx:effect_chance(0.1) then
        ctx:change_stat(pokemon, Stat.Speed, -1)
      end
    end
  end
end}

move.id = "bubble"
move.name = "Bubble"
move.type = Type.Water
move.category = Category.Special
move.pp = 30
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"sheer_force"}
move.base_power = 40
return move
