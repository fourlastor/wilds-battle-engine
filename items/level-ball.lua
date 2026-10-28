local function modifier(ctx, target)
    local user, foe = ctx.user.level, target.level
    if user > 4 * foe then return 8 elseif user > 2 * foe then return 4 elseif user > foe then return 2 end
    return 1
end

return {
  id = "level-ball", name = "Level Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
