-- Move 777: identity and parameters from pokemon_data/assets/moves/777.json.
return {
  id = "decorate", name = "Decorate",
  type = Type.Fairy, category = Category.Status,
  pp = 15, accuracy = Accuracy.Always, priority = 0,
  -- A selected Pokémon may be an ally or an opponent, but never the user.
  target = Target.AnyOther,
  script = function(ctx)
    if not ctx:try_hit(ctx.target, {ignore_protect = true}) then return end
    local attack = ctx:change_stat(ctx.target, Stat.Attack, 2)
    local special = ctx:change_stat(ctx.target, Stat.SpAttack, 2)
    if attack == 0 and special == 0 then return ctx:fail() end
  end,
}
