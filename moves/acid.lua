-- Move 51: Acid. Identity and parameters: pokemon_data/assets/moves/51.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(40, {target = pokemon})
    if hit.hit then
      connected = true
      if ctx:effect_chance(0.1) then
        ctx:change_stat(pokemon, Stat.SpDefense, -1)
      end
    end
  end
end}

move.id = "acid"
move.name = "Acid"
move.type = Type.Poison
move.category = Category.Special
move.pp = 30
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"sheer_force"}
move.base_power = 40
return move
