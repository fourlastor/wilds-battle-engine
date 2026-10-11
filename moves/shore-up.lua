-- Move 659: Shore Up. Identity and parameters: pokemon_data/assets/moves/659.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.User,
    script = function(ctx)
      if ctx.user.hp == ctx.user.max_hp then return ctx:fail() end
      ctx:heal(ctx.user, {fraction = ctx.weather == Weather.Sandstorm and 2/3 or 1/2})
    end}

move.id = "shore-up"
move.name = "Shore Up"
move.type = Type.Ground
move.category = Category.Status
move.pp = 5
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch", "heal"}
return move
