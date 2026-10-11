-- Move 26: Jump Kick. Identity and parameters: pokemon_data/assets/moves/26.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {accuracy = 0.9, flags = {"contact"},
    script = function(ctx)
      local hit = ctx:damage(100)
      if not hit.hit then
        ctx:message(ctx.user.name .. " kept going and crashed!")
        ctx:hurt(ctx.user, {fraction = 1/2})
      end
    end}

move.id = "jump-kick"
move.name = "Jump Kick"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "gravity", "crash_damage"}
move.base_power = 100
return move
