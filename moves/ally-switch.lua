-- Move 502. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "ally-switch", name = "Ally Switch", type = Type.Psychic,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 2, target = Target.User,
  flags = {},
  script = function(ctx)
    local width = ctx.user.team.active_slots
    if width < 2 or width > 3 or (width == 3 and ctx.user.position == 1) then ctx:fail(); return end
    local wanted = ctx.user.position == 0 and width - 1 or 0
    for _, ally in ipairs(ctx.allies) do
      if ally.position == wanted and ctx:swap_positions(ctx.user, ally) then return end
    end
    ctx:fail()
  end,
}
