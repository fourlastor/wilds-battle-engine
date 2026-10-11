-- Move 48: Supersonic. Identity and parameters: pokemon_data/assets/moves/48.json.
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

move.id = "supersonic"
move.name = "Supersonic"
move.type = Type.Normal
move.category = Category.Status
move.pp = 20
move.accuracy = 0.55
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "sound"}
return move
