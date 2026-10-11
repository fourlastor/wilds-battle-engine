-- Move 802: Misty Explosion. Identity and parameters: pokemon_data/assets/moves/802.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local function grounded(who)
  -- The engine's grounded field means explicitly grounded by a rule, not naturally on land.
  return who.grounded or (not who:has_type(Type.Flying) and who.ability ~= "levitate"
    and (not who.item_active or (who.item ~= "air-balloon" and who.item ~= "air_balloon")) and who.hidden ~= Hidden.Air and not who.airborne)
end

local move = {script = function(ctx)
ctx:damage(grounded(ctx.user) and ctx.field.effects.misty_terrain and 150 or 100)
ctx:faint_user()
end}

move.id = "misty-explosion"
move.name = "Misty Explosion"
move.type = Type.Fairy
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.AllOthers
move.flags = {}
move.base_power = 100
return move
