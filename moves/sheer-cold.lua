-- Move 329: Sheer Cold. Identity and parameters: pokemon_data/assets/moves/329.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if ctx.target:has_type(Type.Ice) then return ctx:fail() end
    local accuracy = ctx.user:has_type(Type.Ice) and 0.3 or 0.2
    if not ctx:try_hit(ctx.target, {ohko_accuracy = accuracy}) then return end
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
move.flags = {}
return move
