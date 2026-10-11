-- Move 365: Pluck. Identity and parameters: pokemon_data/assets/moves/365.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Berry removal and consumption history are recorded, but berry activation and resistance-berry timing require an item engine.

local move = {script = function(ctx)
local hit = ctx:damage(60)
if hit.hit and ctx.target.item and ctx.target.item:find("berry", 1, true) then
  local item = ctx:take_item(ctx.target)
  if not item then return end
  ctx:mark(ctx.user, "ate_berry")
  ctx:message(ctx.user.name .. " ate " .. item .. "!")
end
end}

move.id = "pluck"
move.name = "Pluck"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
