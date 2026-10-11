-- Move 166. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "sketch", name = "Sketch", type = Type.Normal,
  category = Category.Status, pp = 1, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target, {ignore_protect=true}) then return end
    local move = ctx.target.last_move
    if ctx.user.transformed or not move or move == "chatter" or move == "sketch" or move == "struggle" then ctx:fail(); return end
    if not ctx:replace_move(ctx.user, "sketch", move, true) then ctx:fail() end
  end,
}
