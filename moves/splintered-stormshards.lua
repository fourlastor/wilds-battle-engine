-- Move 727: Splintered Stormshards. Identity and parameters: pokemon_data/assets/moves/727.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.

local move = {
    script = function(ctx)
      ctx:damage(190)
      if ctx:end_group(ctx.field, "terrain") then ctx:message("The terrain returned to normal.") end
    end}

move.id = "splintered-stormshards"
move.name = "Splintered Stormshards"
move.type = Type.Rock
move.category = Category.Physical
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
