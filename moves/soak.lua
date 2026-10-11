-- Move 487. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "soak", name = "Soak", type = Type.Water,
  category = Category.Status, pp = 20, accuracy = 1.0,
  priority = 0, target = Target.AnyOther,
  flags = {"reflectable"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target) then return end
    local target = ctx.target
    local species = string.lower(target.species):match("^[^-_]+")
    if target.species_id == 493 or target.species_id == 773 or species == "arceus" or species == "silvally"
        or (#target.types == 1 and target:has_type(Type.Water)) then ctx:fail(); return end
    ctx:set_types(target, {Type.Water})
  end,
}
