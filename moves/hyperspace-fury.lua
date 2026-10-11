-- Move 621: Hyperspace Fury. Identity and parameters: pokemon_data/assets/moves/621.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if ctx.user.species ~= "hoopa-unbound" and ctx.user.species ~= "hoopa_unbound" then return ctx:fail() end
    ctx:break_protect(ctx.target)
    if ctx:damage(100, {ignore_protect = true}).hit then ctx:change_stat(ctx.user, Stat.Defense, -1) end
  end,
}

move.id = "hyperspace-fury"
move.name = "Hyperspace Fury"
move.type = Type.Dark
move.category = Category.Physical
move.pp = 5
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"bypass_substitute"}
return move
