from unification import Var


class VolpeVar(Var):
    def __deepcopy__(self, _):
        return VolpeVar()


def var():
    return VolpeVar()
