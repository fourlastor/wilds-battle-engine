-- Move 499: Clear Smog. Identity and parameters: pokemon_data/assets/moves/499.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {accuracy = Accuracy.Always,
    script = function(ctx)
      if ctx:damage(50).hit and ctx:reset_stats(ctx.target) then ctx:message(ctx.target.name .. "'s stat changes were removed!") end
    end}

move.id = "clear-smog"
move.name = "Clear Smog"
move.type = Type.Poison
move.category = Category.Special
move.pp = 15
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
