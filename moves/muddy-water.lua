-- Move 330: Muddy Water. Identity and parameters: pokemon_data/assets/moves/330.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(90, {target = pokemon})
    if hit.hit then
      connected = true
      if ctx:effect_chance(0.3) then
        ctx:change_stat(pokemon, Stat.Accuracy, -1)
      end
    end
  end
end}

move.id = "muddy-water"
move.name = "Muddy Water"
move.type = Type.Water
move.category = Category.Special
move.pp = 10
move.accuracy = 0.85
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
move.base_power = 90
return move
