-- Move 79: Sleep Powder. Identity and parameters: pokemon_data/assets/moves/79.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if ctx.target:has_type(Type.Grass) then return ctx:fail() end
    if ctx:reached() then ctx:apply_status(ctx.target, Status.Asleep) end
  end,
}

move.id = "sleep-powder"
move.name = "Sleep Powder"
move.type = Type.Grass
move.category = Category.Status
move.pp = 15
move.accuracy = 0.75
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "powder"}
return move
