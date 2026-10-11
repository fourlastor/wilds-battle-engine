-- Move 324: Signal Beam. Identity and parameters: pokemon_data/assets/moves/324.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(75, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:confuse(pokemon, {chance = 0.1})
    end
  end
end}

move.id = "signal-beam"
move.name = "Signal Beam"
move.type = Type.Bug
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 75
return move
