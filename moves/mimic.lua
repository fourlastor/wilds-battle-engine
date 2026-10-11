local forbidden = {behemothbash=true, behemothblade=true, chatter=true, dynamaxcannon=true, mimic=true, sketch=true, struggle=true, transform=true}
-- Move 102. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "mimic", name = "Mimic", type = Type.Normal,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local move = ctx.target.last_move
    if ctx.user.transformed or not move or forbidden[move:gsub("[^%w]", ""):lower()] then ctx:fail(); return end
    if not ctx:replace_move(ctx.user, "mimic", move) then ctx:fail() end
  end,
}
