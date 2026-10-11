-- Move 783: identity and parameters from pokemon_data/assets/moves/783.json.
return {
  flags = {"sheer_force"},
  base_power = 110,
  id = "aura-wheel", name = "Aura Wheel",
  type = Type.Electric, category = Category.Physical,
  pp = 10, accuracy = 1, priority = 0, target = Target.Selected,
  script = function(ctx)
    local species = ctx.user.species
    if ctx.user.species_id ~= 877 and species ~= "morpeko"
        and species ~= "morpeko-full-belly" and species ~= "morpeko-hangry" then
      return ctx:fail()
    end
    local kind = (ctx.user.form or species) == "morpeko-hangry" and Type.Dark or Type.Electric
    if ctx:damage(110, {type = kind}).hit and ctx.user.hp > 0 then
      ctx:change_stat(ctx.user, Stat.Speed, 1)
    end
  end,
}
