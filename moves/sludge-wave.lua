-- Move 482: Sludge Wave. Identity and parameters: pokemon_data/assets/moves/482.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(95, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:apply_status(pokemon, Status.Poisoned, {chance = 0.1})
    end
  end
end}

move.id = "sludge-wave"
move.name = "Sludge Wave"
move.type = Type.Poison
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {}
return move
