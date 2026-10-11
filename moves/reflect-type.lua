-- Move 513. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "reflect-type", name = "Reflect Type", type = Type.Normal,
  category = Category.Status, pp = 15, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local species = string.lower(ctx.user.species):match("^[^-_]+")
    if ctx.user.species_id == 493 or ctx.user.species_id == 773 or species == "arceus" or species == "silvally" then ctx:fail(); return end
    local types = {}
    for _, kind in ipairs(ctx.target.base_types) do
      if kind ~= Type.None and not (kind == Type.Flying and ctx.target.effects.roosting) then types[#types + 1] = kind end
    end
    if #types == 0 then
      if ctx.target.added_type then types = {Type.Normal} else ctx:fail(); return end
    end
    local added = ctx.target.added_type
    ctx:set_types(ctx.user, types)
    if added then ctx:add_type(ctx.user, added) end
  end,
}
