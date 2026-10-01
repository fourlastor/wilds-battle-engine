return { id="explosion", name="Explosion", type=Type.Normal, category=Category.Physical, pp=5, target=Target.AllOthers,
    script=function(ctx)
      ctx:faint_user()
      ctx:damage(250)
    end }
