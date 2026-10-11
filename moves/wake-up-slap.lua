-- Move 358: Wake-Up Slap. Identity and parameters: pokemon_data/assets/moves/358.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx)
      local asleep = ctx.target.asleep
      if ctx:damage(asleep and 140 or 70).hit and asleep then ctx:cure_status(ctx.target, Status.Asleep) end
    end}

move.id = "wake-up-slap"
move.name = "Wake-Up Slap"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
move.base_power = 70
return move
