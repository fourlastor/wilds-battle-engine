-- Move 186: Sweet Kiss. Identity and parameters: pokemon_data/assets/moves/186.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
  local connected = false
  for _, pokemon in ipairs(ctx.targets) do
    if ctx:reached(pokemon) then
      connected = true
      ctx:confuse(pokemon, {chance = 1})
    end
  end
  if not connected then return ctx:fail() end
end}

move.id = "sweet-kiss"
move.name = "Sweet Kiss"
move.type = Type.Fairy
move.category = Category.Status
move.pp = 10
move.accuracy = 0.75
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable"}
return move
