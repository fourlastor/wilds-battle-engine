-- Move 174. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "curse", name = "Curse", type = Type.Ghost,
  category = Category.Status, pp = 10, accuracy = Accuracy.Always,
  priority = 0, target = Target.RandomOpponent,
  flags = {"bypass_substitute", "adjacent_target"},
  target_user_unless_type = Type.Ghost,
  script = function(ctx)
    if ctx.user:has_type(Type.Ghost) then
      if not ctx:try_hit(ctx.target, {ignore_protect = true}) then return end
      if not ctx:start_effect(ctx.target, "curse", {rules = {
        {kind = Rule.DamageEachTurn, fraction = 1/4, message = "{name} is afflicted by the curse!"}
      }}) then ctx:fail(); return end
      ctx:hurt(ctx.user, math.max(1, math.floor(ctx.user.max_hp / 2)))
    else
      local changed = false
      for _, entry in ipairs({{Stat.Speed, -1}, {Stat.Attack, 1}, {Stat.Defense, 1}}) do
        if ctx:change_stat(ctx.user, entry[1], entry[2]) ~= 0 then changed = true end
      end
      if not changed then ctx:fail() end
    end
  end,
}
