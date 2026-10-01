return { id="triple_kick", name="Triple Kick", type=Type.Fighting, category=Category.Physical, pp=10,
    script=function(ctx)
      for hit=1,3 do
        local result = ctx:damage(10*hit, {accuracy=0.9})
        if not result.hit then return end
      end
    end }
