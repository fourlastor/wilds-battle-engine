return { id="charge", name="Charge", type=Type.Electric, category=Category.Status, pp=20,
    target=Target.User, accuracy=Accuracy.Always,
    script=function(ctx)
      ctx:change_self_stat(ctx.Stat.SpDefense, 1)
      ctx:boost_next_move(ctx.Type.Electric, 2)
    end }
