-- Move 355: Roost. Identity and parameters: pokemon_data/assets/moves/355.json.
-- Behavior checked against PokeAPI; see catalog.json for scope and limitations.

local move = {target = Target.User,
    script = function(ctx)
      if ctx.user.hp == ctx.user.max_hp then return ctx:fail() end
      ctx:heal_self(1/2)
      ctx:start_effect(ctx.user, "roosting", {turns = 1, rules = {{kind = Rule.WithoutType, type = Type.Flying}}})
    end}

move.id = "roost"
move.name = "Roost"
move.type = Type.Flying
move.category = Category.Status
move.pp = 5
move.accuracy = Accuracy.Always
move.priority = 0
move.target = Target.User
move.flags = {"snatch", "heal"}
return move
