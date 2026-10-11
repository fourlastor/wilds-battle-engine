-- Move 593: Hyperspace Hole. Identity and parameters: pokemon_data/assets/moves/593.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {accuracy = Accuracy.Always,
    script = function(ctx)
      ctx:break_protect(ctx.target)
      ctx:damage(80, {ignore_protect = true})
    end}

move.id = "hyperspace-hole"
move.name = "Hyperspace Hole"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 5
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {}
return move
