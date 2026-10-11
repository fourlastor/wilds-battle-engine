-- Move 160. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "conversion", name = "Conversion", type = Type.Normal,
  category = Category.Status, pp = 30, accuracy = Accuracy.Always,
  priority = 0, target = Target.User,
  flags = {"snatch"},
  script = function(ctx)
    local user = ctx.user
    local species = string.lower(user.species):match("^[^-_]+")
    local move = ctx:move_info(user.moves[1])
    if not move or user:has_type(move.type) or user.species_id == 493 or user.species_id == 773
        or species == "arceus" or species == "silvally" then ctx:fail(); return end
    ctx:set_types(user, {move.type})
  end,
}
