-- Move 93: Confusion. Identity and parameters: pokemon_data/assets/moves/93.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    local hit = ctx:damage(50, {target = pokemon})
    if hit.hit then
      connected = true
      ctx:confuse(pokemon, {chance = 0.1})
    end
  end
end}

move.id = "confusion"
move.name = "Confusion"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.base_power = 50
return move
