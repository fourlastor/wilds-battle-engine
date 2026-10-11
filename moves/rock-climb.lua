-- Move 431: Rock Climb. Identity and parameters: pokemon_data/assets/moves/431.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(90, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:confuse(pokemon, {chance = 0.2})
    end
  end
end}

move.id = "rock-climb"
move.name = "Rock Climb"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 20
move.accuracy = 0.85
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
