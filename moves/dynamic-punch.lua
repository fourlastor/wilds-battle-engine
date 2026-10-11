-- Move 223: Dynamic Punch. Identity and parameters: pokemon_data/assets/moves/223.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(100, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:confuse(pokemon, {chance = 1})
    end
  end
end}

move.id = "dynamic-punch"
move.name = "Dynamic Punch"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 5
move.accuracy = 0.5
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "punch"}
move.base_power = 100
return move
