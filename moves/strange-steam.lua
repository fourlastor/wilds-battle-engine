-- Move 790: Strange Steam. Identity and parameters: pokemon_data/assets/moves/790.json.
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

move.id = "strange-steam"
move.name = "Strange Steam"
move.type = Type.Fairy
move.category = Category.Special
move.pp = 10
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
