-- Move 826: Eerie Spell. Identity and parameters: pokemon_data/assets/moves/826.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {script = function(ctx)
local hit = ctx:damage(80)
if hit.hit and ctx.target.hp > 0 and ctx.target.last_move then
  local lost = ctx:reduce_pp(ctx.target, ctx.target.last_move, 3)
  if lost > 0 then ctx:message(ctx.target.name .. " lost " .. lost .. " PP from its last move!") end
end
end}

move.id = "eerie-spell"
move.name = "Eerie Spell"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sound"}
return move
