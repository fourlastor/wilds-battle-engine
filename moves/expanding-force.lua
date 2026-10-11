-- Move 797: Expanding Force. Identity and parameters: pokemon_data/assets/moves/797.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local function grounded(who)
  -- The engine's grounded field means explicitly grounded by a rule, not naturally on land.
  return who.grounded or (not who:has_type(Type.Flying) and who.ability ~= "levitate"
    and (not who.item_active or (who.item ~= "air-balloon" and who.item ~= "air_balloon")) and who.hidden ~= Hidden.Air and not who.airborne)
end

local move = {script = function(ctx)
if grounded(ctx.user) and ctx.field.effects.psychic_terrain then
  for _, pokemon in ipairs(ctx.foes) do ctx:damage(120, {target = pokemon}) end
else ctx:damage(80) end
end}

move.id = "expanding-force"
move.name = "Expanding Force"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
