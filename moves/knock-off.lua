-- Move 282: Knock Off. Identity and parameters: pokemon_data/assets/moves/282.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Species-bound unremovable items require host item rules; ordinary items and Sticky Hold are handled.

local move = {script = function(ctx)
local hit = ctx:damage(ctx.target.item and 97 or 65)
if hit.hit and ctx.target.item and ctx.target.ability ~= "sticky-hold" and ctx.target.ability ~= "sticky_hold" then
  ctx:take_item(ctx.target)
end
end}

move.id = "knock-off"
move.name = "Knock Off"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 20
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "incomplete"}
return move
