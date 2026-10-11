-- Move 712: Spectral Thief. Identity and parameters: pokemon_data/assets/moves/712.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx)
      -- The boosts are taken first, so that the hit already has them; the hit follows the same check.
      if ctx:reached() then ctx:steal_boosts(ctx.target, ctx.user) end
      ctx:damage(90)
    end}

move.id = "spectral-thief"
move.name = "Spectral Thief"
move.type = Type.Ghost
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "bypass_substitute"}
return move
