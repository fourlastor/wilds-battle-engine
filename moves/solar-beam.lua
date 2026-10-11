return { id="solar_beam", name="Solar Beam", type=Type.Grass, category=Category.Special, pp=10,
    manual_announce=true,
    script=function(ctx)
      if ctx.turn == 1 and ctx.weather ~= Weather.Sun then
        ctx:message(ctx.user.name .. " took in sunlight!")
        ctx:force_move(2, ctx.TargetPolicy.SameTarget)
        return
      end
      ctx:announce()
      ctx:damage(120)
    end }
