-- Move 555: Snarl. Identity and parameters: pokemon_data/assets/moves/555.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(55, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:change_stat(pokemon, Stat.SpAttack, -1)
    end
  end
end}

move.id = "snarl"
move.name = "Snarl"
move.type = Type.Dark
move.category = Category.Special
move.pp = 15
move.accuracy = 0.95
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"sound", "sheer_force"}
move.base_power = 55
return move
