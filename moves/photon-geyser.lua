-- Move 722: Photon Geyser. Identity and parameters: pokemon_data/assets/moves/722.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {
    script = function(ctx)
      local physical = ctx.user.attack > ctx.user.sp_attack
      ctx:damage(100, {category = physical and Category.Physical or Category.Special})
    end}

move.id = "photon-geyser"
move.name = "Photon Geyser"
move.type = Type.Psychic
move.category = Category.Special
move.pp = 5
move.accuracy = 1
move.priority = 0
move.target = Target.Selected
move.flags = {"ignore_ability"}
return move
