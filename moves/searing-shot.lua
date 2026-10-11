-- Move 545: Searing Shot. Identity and parameters: pokemon_data/assets/moves/545.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(100, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:apply_status(pokemon, Status.Burned, {chance = 0.3})
    end
  end
end}

move.id = "searing-shot"
move.name = "Searing Shot"
move.type = Type.Fire
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {"bullet"}
return move
