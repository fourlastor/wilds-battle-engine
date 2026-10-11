-- Move 723: Light That Burns the Sky. Identity and parameters: pokemon_data/assets/moves/723.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.
-- INCOMPLETE: Z-Move activation, species/item eligibility, once-per-battle limits and partial Protect damage require the host.
-- INCOMPLETE: The engine does not execute abilities; explicit ability-bypass semantics require an ability engine.

local move = {
    script = function(ctx)
      local physical = ctx.user.attack > ctx.user.sp_attack
      ctx:damage(200, {category = physical and Category.Physical or Category.Special})
    end}

move.id = "light-that-burns-the-sky"
move.name = "Light That Burns the Sky"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 1
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.Selected
move.flags = {"incomplete"}
return move
