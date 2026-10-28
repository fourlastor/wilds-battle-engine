return {
  base_power = 120, flags={"charge"}, id="solar-beam", name="Solar Beam", type=Type.Grass, category=Category.Special, pp=10,
    manual_announce=true,
    script=function(ctx)
      if ctx.turn == 1 and ctx.weather ~= Weather.Sun then
        ctx:message("{0} took in sunlight!", ctx.user)
        ctx:force_move(2, ctx.TargetPolicy.SameTarget)
        return
      end
      ctx:announce()
      ctx:damage(120)
    end }
