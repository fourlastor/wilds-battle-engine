-- Move 822: Fiery Wrath. Identity and parameters: pokemon_data/assets/moves/822.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(90, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:flinch(pokemon, {chance = 0.2})
    end
  end
end}

move.id = "fiery-wrath"
move.name = "Fiery Wrath"
move.type = Type.Dark
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.AllOpponents
move.flags = {}
return move
