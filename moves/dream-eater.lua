return {
  base_power = 100, flags = {"heal"}, id="dream_eater", name="Dream Eater", type=Type.Psychic, category=Category.Special, pp=15,
    script=function(ctx)
      if not ctx.target.asleep then return ctx:fail() end
      ctx:damage(100, {drain=0.5})
    end }
