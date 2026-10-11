-- Move 562: Belch. Identity and parameters: pokemon_data/assets/moves/562.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Fails on execution until the ate_berry mark is set. Pre-selection legality and automatic held-berry consumption require engine/host support.

local move = {accuracy = 0.9,
    script = function(ctx)
      if not ctx.user.marks.ate_berry then return ctx:fail() end
      ctx:damage(120)
    end}

move.id = "belch"
move.name = "Belch"
move.type = Type.Poison
move.category = Category.Special
move.pp = 10
move.accuracy = 0.9
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
