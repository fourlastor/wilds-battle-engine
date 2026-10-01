return { id="rage", name="Rage", type=Type.Normal, category=Category.Physical, pp=20,
    on_hit=function(ctx)
      ctx:change_self_stat(ctx.Stat.Attack, 1)
    end,
    script=function(ctx)
      ctx:watch_hits_until_next_action()
      ctx:damage(20)
    end }
