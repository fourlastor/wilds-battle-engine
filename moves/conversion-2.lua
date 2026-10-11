-- Move 176. PokeAPI identity; Gen 8 behavior (see catalog.json).
return {
  id = "conversion-2", name = "Conversion 2", type = Type.Normal,
  category = Category.Status, pp = 30, accuracy = Accuracy.Always,
  priority = 0, target = Target.AnyOther,
  flags = {"bypass_substitute"},
  script = function(ctx)
    if not ctx:try_hit(ctx.target, {ignore_protect=true}) then return end
    local user = ctx.user
    local species = string.lower(user.species):match("^[^-_]+")
    if not ctx.target.last_move_type or user.species_id == 493 or user.species_id == 773
        or species == "arceus" or species == "silvally" then ctx:fail(); return end
    local resists = {
      [Type.Normal]={Type.Rock,Type.Steel,Type.Ghost},
      [Type.Fighting]={Type.Poison,Type.Flying,Type.Psychic,Type.Bug,Type.Fairy,Type.Ghost},
      [Type.Flying]={Type.Electric,Type.Rock,Type.Steel},
      [Type.Poison]={Type.Poison,Type.Ground,Type.Rock,Type.Ghost,Type.Steel},
      [Type.Ground]={Type.Grass,Type.Bug,Type.Flying},
      [Type.Rock]={Type.Fighting,Type.Ground,Type.Steel},
      [Type.Bug]={Type.Fire,Type.Fighting,Type.Poison,Type.Flying,Type.Ghost,Type.Steel,Type.Fairy},
      [Type.Ghost]={Type.Dark,Type.Normal},
      [Type.Steel]={Type.Fire,Type.Water,Type.Electric,Type.Steel},
      [Type.Fire]={Type.Fire,Type.Water,Type.Rock,Type.Dragon},
      [Type.Water]={Type.Water,Type.Grass,Type.Dragon},
      [Type.Grass]={Type.Fire,Type.Grass,Type.Poison,Type.Flying,Type.Bug,Type.Dragon,Type.Steel},
      [Type.Electric]={Type.Electric,Type.Grass,Type.Dragon,Type.Ground},
      [Type.Psychic]={Type.Psychic,Type.Steel,Type.Dark},
      [Type.Ice]={Type.Fire,Type.Water,Type.Ice,Type.Steel},
      [Type.Dragon]={Type.Steel,Type.Fairy},
      [Type.Dark]={Type.Fighting,Type.Dark,Type.Fairy},
      [Type.Fairy]={Type.Fire,Type.Poison,Type.Steel},
    }
    local choices = {}
    for _, kind in ipairs(resists[ctx.target.last_move_type] or {}) do
      if not user:has_type(kind) then choices[#choices+1] = kind end
    end
    if #choices == 0 then ctx:fail(); return end
    ctx:set_types(user, {choices[ctx:random_int(1,#choices)]})
  end,
}
