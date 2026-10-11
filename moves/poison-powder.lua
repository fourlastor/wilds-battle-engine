-- Move 77: Poison Powder. Identity and parameters: pokemon_data/assets/moves/77.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if ctx.target:has_type(Type.Grass) then return ctx:fail() end
    if ctx:reached() then ctx:apply_status(ctx.target, Status.Poisoned) end
  end,
}

move.id = "poison-powder"
move.name = "Poison Powder"
move.type = Type.Poison
move.category = Category.Status
move.pp = 35
move.accuracy = 0.75
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "powder"}
return move
