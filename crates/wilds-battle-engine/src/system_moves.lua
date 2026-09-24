-- Universal fallback. This move is not part of a Pokémon's learned moveset.
return {
  id="struggle", name="Struggle", type=Type.Normal, category=Category.Physical,
  pp=1, target=Target.RandomOpponent, accuracy=Accuracy.Always, auto_only=true,
  script=function(ctx)
    local result = ctx:damage(50, {typeless=true})
    if result.hit then ctx:recoil_max_hp(0.25) end
  end,
}
