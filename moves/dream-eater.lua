return { id="dream_eater", name="Dream Eater", type=Type.Psychic, category=Category.Special, pp=15,
    script=function(ctx)
      if ctx.target.status ~= ctx.Status.Asleep then return ctx:fail() end
      ctx:damage(100, {drain=0.5})
    end }
