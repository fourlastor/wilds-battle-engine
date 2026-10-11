-- Move 252: Fake Out. Identity and parameters: pokemon_data/assets/moves/252.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {priority = 3, flags = {"contact", "flinch_secondary", "sheer_force"},
    script = function(ctx)
      if ctx.user.turns_on_field > 1 then return ctx:fail() end
      if ctx:damage(40).hit then ctx:flinch(ctx.target) end
    end}

move.id = "fake-out"
move.name = "Fake Out"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 3
move.target = Target.Selected
move.flags = {"contact", "flinch_secondary", "sheer_force"}
move.base_power = 40
return move
