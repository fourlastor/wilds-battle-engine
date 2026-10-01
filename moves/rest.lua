return { id="rest", name="Rest", type=Type.Psychic, category=Category.Status, pp=5,
    target=Target.User, accuracy=Accuracy.Always, fail_on_full_hp=true,
    effects={{kind=Effect.Status, status=Status.RestSleep, replace=true}, {kind=Effect.Heal, fraction=1, hide_message=true}} }
