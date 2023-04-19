def __VERIFIER_assert(a):
    pass

def __VERIFIER_assume(a):
    pass

def main(a: int, b: int, c: int, d: int):
    """
    pre: True
    """
    unreachable: bool = True

    if c & 1 == d & 16:
        a = c
        b = d
        for i in range(100):
            if a & 1 != b & 16:
                unreachable = False
                assert False
    __VERIFIER_assert(unreachable)

