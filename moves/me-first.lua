local forbidden = {beakblast=true, belch=true, blazingtorque=true, chatter=true, combattorque=true, comeuppance=true, counter=true, covet=true, focuspunch=true, magicaltorque=true, mefirst=true, metalburst=true, mirrorcoat=true, noxioustorque=true, shelltrap=true, struggle=true, thief=true, wickedtorque=true}
-- Move 382. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "me-first", name = "Me First", type = Type.Normal,
  category = Category.Status, pp = 20, accuracy = Accuracy.Always,
  priority = 0, target = Target.Selected,
  flags = {"bypass_substitute", "calls_move", "adjacent_target"},
  script = function(ctx)
    local pending = false
    for _, pokemon in ipairs(ctx.pending) do if pokemon == ctx.target then pending=true; break end end
    local move = ctx.target.selected_move
    if not pending or ctx.target.recharging or not move or not move.damaging or forbidden[move.id:gsub("[^%w]", ""):lower()] then ctx:fail(); return end
    if not ctx:try_hit(ctx.target) then return end
    ctx:start_effect(ctx.user, "me_first", {turns=1, rules={{kind=Rule.BasePower, factor=1.5}}})
    ctx:call_move(move.id, ctx.target)
    ctx:end_effect(ctx.user, "me_first")
  end,
}
