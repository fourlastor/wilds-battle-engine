return { id="hyper_beam", name="Hyper Beam", type=Type.Normal, category=Category.Special, pp=5, accuracy=0.9,
    script=function(ctx)
      local result = ctx:damage(150)
      if result.hit then ctx:recharge() end
    end }
