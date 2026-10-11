-- Move 560: Flying Press. Identity and parameters: pokemon_data/assets/moves/560.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {accuracy = 0.95, flags = {"contact"},
    script = function(ctx)
      local small = ctx.target.marks.minimized ~= nil
      ctx:damage(small and 200 or 100, {also_type = Type.Flying, never_miss = small})
    end}

move.id = "flying-press"
move.name = "Flying Press"
move.type = Type.Fighting
move.category = Category.Physical
move.pp = 10
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {"contact", "gravity"}
move.base_power = 100
return move
