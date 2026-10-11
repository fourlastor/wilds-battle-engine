-- Move 565: Fell Stinger. Identity and parameters: pokemon_data/assets/moves/565.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {flags = {"contact"},
    script = function(ctx)
      if ctx:damage(50).fainted then ctx:change_stat(ctx.user, Stat.Attack, 3) end
    end}

move.id = "fell-stinger"
move.name = "Fell Stinger"
move.type = Type.Bug
move.category = Category.Physical
move.pp = 25
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"contact"}
return move
