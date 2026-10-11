-- Move 496: Round. Identity and parameters: pokemon_data/assets/moves/496.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"sound"},
    script = function(ctx)
      ctx:damage(ctx.field.marks.round and 120 or 60)
      ctx:mark(ctx.field, "round", 1, 1)
      ctx:hurry()
    end}

move.id = "round"
move.name = "Round"
move.type = Type.Normal
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"sound"}
return move
