-- Move 147: Spore. Identity and parameters: pokemon_data/assets/moves/147.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if ctx.target:has_type(Type.Grass) then return ctx:fail() end
    if ctx:reached() then ctx:apply_status(ctx.target, Status.Asleep) end
  end,
}

move.id = "spore"
move.name = "Spore"
move.type = Type.Grass
move.category = Category.Status
move.pp = 15
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "powder"}
return move
