-- Move 78: Stun Spore. Identity and parameters: pokemon_data/assets/moves/78.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if ctx.target:has_type(Type.Grass) then return ctx:fail() end
    if ctx:reached() then ctx:apply_status(ctx.target, Status.Paralyzed) end
  end,
}

move.id = "stun-spore"
move.name = "Stun Spore"
move.type = Type.Grass
move.category = Category.Status
move.pp = 30
move.accuracy = 0.75
move.priority = 0
move.target = Target.Selected
move.flags = {"reflectable", "powder"}
return move
