-- Move 130: Skull Bash. Identity and parameters: pokemon_data/assets/moves/130.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {manual_announce = true,
    script = function(ctx)
      if ctx.turn == 1 then
        ctx:message("{0} tucked in its head!", ctx.user)
        ctx:change_stat(ctx.user, Stat.Defense, 1)
        ctx:force_move(2, TargetPolicy.SameTarget)
        return
      end
      ctx:announce()
      ctx:damage(130)
    end}

move.id = "skull-bash"
move.name = "Skull Bash"
move.type = Type.Normal
move.category = Category.Physical
move.pp = 10
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"charge", "contact"}
move.base_power = 130
return move
