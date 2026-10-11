-- Move 592: Steam Eruption. Identity and parameters: pokemon_data/assets/moves/592.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
  script = function(ctx)
    if ctx:damage(110).hit then
      ctx:cure_status(ctx.target, Status.Frozen)
      ctx:apply_status(ctx.target, Status.Burned, {chance = 0.3})
    end
  end,
}

move.id = "steam-eruption"
move.name = "Steam Eruption"
move.type = Type.Water
move.category = Category.Special
move.pp = 5
move.accuracy = 0.95
move.priority = 0
move.target = Target.Selected
move.flags = {}
move.usable_while_frozen = true
move.base_power = 110
return move
