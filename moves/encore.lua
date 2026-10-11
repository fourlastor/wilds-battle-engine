local forbidden = {assist=true, blazingtorque=true, combattorque=true, copycat=true, dynamaxcannon=true, encore=true, magicaltorque=true, mefirst=true, metronome=true, mimic=true, mirrormove=true, naturepower=true, noxioustorque=true, sketch=true, sleeptalk=true, struggle=true, transform=true, wickedtorque=true}
-- Move 227. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "encore", name = "Encore", type = Type.Normal,
  category = Category.Status, pp = 5, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute", "reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local move = ctx.target.last_move
    if not move or forbidden[move:gsub("[^%w]", ""):lower()] or not ctx.target.pp[move] or ctx.target.pp[move] == 0 or ctx.target.effects.encore then ctx:fail(); return end
    local turns = 4
    for _, pokemon in ipairs(ctx.pending) do if pokemon == ctx.target then turns=3; break end end
    ctx:start_effect(ctx.target, "encore", {turns=turns, rules={{kind=Rule.OnlyMove, id=move}}})
  end,
}
