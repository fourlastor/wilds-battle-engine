-- Move 235: Synthesis. Identity and parameters: pokemon_data/assets/moves/235.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.User,
    script = function(ctx)
      if ctx.user.hp == ctx.user.max_hp then return ctx:fail() end
      local share = (ctx.weather == nil or ctx.weather == Weather.StrongWinds) and 1/2 or ctx.weather == Weather.Sun and 2/3 or 1/4
      ctx:heal(ctx.user, {fraction = share})
      ctx:message(ctx.user.name .. " regained health!")
    end}

move.id = "synthesis"
move.name = "Synthesis"
move.type = Type.Grass
move.category = Category.Status
move.pp = 5
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch", "heal"}
return move
