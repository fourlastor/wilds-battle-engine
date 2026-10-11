-- Move 329: Sheer Cold. Identity and parameters: pokemon_data/assets/moves/329.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Ice immunity and reduced accuracy for non-Ice users work; an AlwaysHit rule cannot be inspected to bypass the extra accuracy adjustment.

local move = {
  script = function(ctx)
    if ctx.target:has_type(Type.Ice) then return ctx:fail() end
    if not ctx:try_hit(ctx.target) then return end
    local chance = math.min(1, 0.3 + (ctx.user.level - ctx.target.level) / 100)
    local adjusted = math.min(1, 0.2 + (ctx.user.level - ctx.target.level) / 100)
    if not ctx.user:has_type(Type.Ice) and not ctx:chance(adjusted / chance) then
      ctx:message(ctx.user.name .. "'s attack missed!")
      return ctx:fail()
    end
    ctx:direct_damage(ctx.target, ctx.target.hp)
  end,
}

move.id = "sheer-cold"
move.name = "Sheer Cold"
move.type = Type.Ice
move.category = Category.Special
move.pp = 5
move.accuracy = Accuracy.Ohko
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
