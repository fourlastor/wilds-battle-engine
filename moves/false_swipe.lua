return { id="false_swipe", name="False Swipe", type=Type.Normal, category=Category.Physical, pp=40,
    script=function(ctx)
      ctx:damage(40, {min_target_hp=1})
    end }
