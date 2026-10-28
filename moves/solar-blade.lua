-- Move 669: Solar Blade. Identity and parameters: pokemon_data/assets/moves/669.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {manual_announce = true, flags = {"contact"},
    script = function(ctx)
      if ctx.turn == 1 and ctx.weather ~= Weather.Sun then
        ctx:message("{0} absorbed light!", ctx.user)
        ctx:force_move(2, TargetPolicy.SameTarget)
        return
      end
      ctx:announce()
      local dim = ctx.weather ~= nil and ctx.weather ~= Weather.Sun
      ctx:damage(dim and 62 or 125)
    end}

move.id = "solar-blade"
move.name = "Solar Blade"
move.type = Type.Grass
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"charge", "contact", "slicing"}
move.base_power = 125
return move
