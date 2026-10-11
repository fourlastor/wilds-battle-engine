-- Move 686: Revelation Dance. Identity and parameters: pokemon_data/assets/moves/686.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx)
      local kind = ctx.user.types[1]
      if kind == nil or kind == Type.None then ctx:damage(90, {typeless = true}) else ctx:damage(90, {type = kind}) end
    end}

move.id = "revelation-dance"
move.name = "Revelation Dance"
move.type = Type.Normal
move.category = Category.Special
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"dance"}
move.base_power = 90
return move
