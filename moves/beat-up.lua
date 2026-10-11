-- Move 251. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "beat-up", name = "Beat Up", type = Type.Dark,
  category = Category.Physical, pp = 10, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {},
  script = function(ctx)
    local powers = {}
    for _, pokemon in ipairs(ctx.user.team.roster) do
      if pokemon == ctx.user or (pokemon.hp > 0 and not pokemon.status) then
        if not pokemon.base_attack then ctx:message("Beat Up requires species base Attack data."); ctx:fail(); return end
        powers[#powers+1] = 5 + math.floor(pokemon.base_attack / 10)
      end
    end
    ctx:multi_hit(powers[1], #powers, #powers, {powers=powers})
  end,
}
