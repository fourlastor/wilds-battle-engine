-- Move 804: Rising Voltage. Identity and parameters: pokemon_data/assets/moves/804.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local function grounded(who)
  -- The engine's grounded field means explicitly grounded by a rule, not naturally on land.
  return who.grounded or (not who:has_type(Type.Flying) and who.ability ~= "levitate"
    and who.item ~= "air-balloon" and who.item ~= "air_balloon" and who.hidden ~= Hidden.Air)
end

local move = {script = function(ctx)
ctx:damage(grounded(ctx.target) and ctx.field.effects.electric_terrain and 140 or 70)
end}

move.id = "rising-voltage"
move.name = "Rising Voltage"
move.type = Type.Electric
move.category = Category.Special
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
