local function modifier(ctx, target)
    local user = ctx.user
    local opposite = user.gender ~= target.gender and user.gender ~= "Genderless" and target.gender ~= "Genderless"
    return (user.species == target.species and opposite) and 8 or 1
end

return {
  id = "love-ball", name = "Love Ball", targets = "foe",
  usable = function(_, ctx) return ctx.wild_battle end,
  script = function(ctx) ctx:attempt_capture(ctx.target, modifier(ctx, ctx.target)) end,
}
