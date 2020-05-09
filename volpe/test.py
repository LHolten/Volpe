from unification import unify, var
from unification.match import Dispatcher, VarDispatcher

from volpe_types import int64, flt64

if __name__ == '__main__':
    d = VarDispatcher('d')
    x = var('x')
    y = var('y')

    d.add((1,), lambda _: 1)
    d.add((2,), lambda _: 2)

    # unify
    print(d(x))
