-- Move 563: Rototiller. Identity and parameters: pokemon_data/assets/moves/563.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local function grounded(who)
  -- The engine's grounded field means explicitly grounded by a rule, not naturally on land.
  return who.grounded or (not who:has_type(Type.Flying) and who.ability ~= "levitate"
    and (not who.item_active or (who.item ~= "air-balloon" and who.item ~= "air_balloon")) and who.hidden ~= Hidden.Air and not who.airborne)
end

local move = {script = function(ctx)
local affected = false
for _, pokemon in ipairs(ctx.everyone) do
  if grounded(pokemon) and pokemon:has_type(Type.Grass) and ctx:reached(pokemon) then
    local attack = ctx:change_stat(pokemon, Stat.Attack, 1)
    local special = ctx:change_stat(pokemon, Stat.SpAttack, 1)
    affected = affected or attack ~= 0 or special ~= 0
  end
end
if not affected then return ctx:fail() end
end}

move.id = "rototiller"
move.name = "Rototiller"
move.type = Type.Ground
move.category = Category.Status
move.pp = 10
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.All
move.flags = {"pressure_all"}
return move
