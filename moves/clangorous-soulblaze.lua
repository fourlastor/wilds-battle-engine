-- Move 728: Clangorous Soulblaze. Identity and parameters: pokemon_data/assets/moves/728.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.

local move = {target = Target.AllOpponents, flags = {"sound"},
    script = function(ctx)
      if not ctx:damage(185).hit then return end
      for _, stat in ipairs({Stat.Attack, Stat.Defense, Stat.SpAttack, Stat.SpDefense, Stat.Speed}) do
        ctx:change_stat(ctx.user, stat, 1)
      end
    end}

move.id = "clangorous-soulblaze"
move.name = "Clangorous Soulblaze"
move.type = Type.Dragon
move.category = Category.Special
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.AllOpponents
move.flags = {"sound", "incomplete"}
return move
