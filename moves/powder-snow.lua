-- Move 181: Powder Snow. Identity and parameters: pokemon_data/assets/moves/181.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(40, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:apply_status(pokemon, Status.Frozen, {chance = 0.1})
    end
  end
end}

move.id = "powder-snow"
move.name = "Powder Snow"
move.type = Type.Ice
move.category = Category.Special
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
return move
