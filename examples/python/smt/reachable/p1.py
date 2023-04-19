def __VERIFIER_assert(a):
    pass

def __VERIFIER_assume(a):
    pass

def main(a: int, b: int, c: int, d: int):
    """
    pre: True
    """
    unreachable: bool = True
    a = a ^ b
    if a < (c // 2):
        for i in range(6):
            d = c + b
            for i in range(3):
                d = d + (b // 2)
        if 0 < ((d > a) & True):
            unreachable = False
            assert False
    __VERIFIER_assert(unreachable)
