-- Move 690: Beak Blast. Identity and parameters: pokemon_data/assets/moves/690.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {priority = -3,
    on_turn_start = function(ctx)
      ctx:message(ctx.user.name .. " started heating up its beak!")
      ctx:watch_contacts_until_next_action()
    end,
    on_hit = function(ctx)
      if ctx.hit.contact then ctx:apply_status(ctx.target, Status.Burned) end
    end,
    script = function(ctx) ctx:damage(100) end}

move.id = "beak-blast"
move.name = "Beak Blast"
move.type = Type.Flying
move.category = Category.Physical
move.pp = 15
move.accuracy = 1
move.priority = -3
move.target = Target.Selected
move.flags = {"bullet"}
return move
