-- Move 146: Dizzy Punch. Identity and parameters: pokemon_data/assets/moves/146.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(70, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:confuse(pokemon, {chance = 0.2})
    end
  end
end}

move.id = "dizzy-punch"
move.name = "Dizzy Punch"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch", "sheer_force"}
move.base_power = 70
return move
