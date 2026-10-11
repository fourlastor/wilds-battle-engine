-- Move 805: Terrain Pulse. Identity and parameters: pokemon_data/assets/moves/805.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local function grounded(who)
  -- The engine's grounded field means explicitly grounded by a rule, not naturally on land.
  return who.grounded or (not who:has_type(Type.Flying) and who.ability ~= "levitate"
    and who.item ~= "air-balloon" and who.item ~= "air_balloon" and who.hidden ~= Hidden.Air and not who.airborne)
end

local move = {script = function(ctx)
local kind
if grounded(ctx.user) then
  local terrains = {electric_terrain=Type.Electric, grassy_terrain=Type.Grass, misty_terrain=Type.Fairy, psychic_terrain=Type.Psychic}
  for terrain, value in pairs(terrains) do if ctx.field.effects[terrain] then kind = value end end
end
ctx:damage(kind and 100 or 50, {type = kind})
end}

move.id = "terrain-pulse"
move.name = "Terrain Pulse"
move.type = Type.Normal
move.category = Category.Special
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"pulse"}
return move
