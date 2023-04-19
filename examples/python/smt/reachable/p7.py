def __VERIFIER_assert(a):
    pass

def __VERIFIER_assume(a):
    pass

def main(a: int, b: int, c: int, d: int):
    """
    pre: True
    """
    unreachable: bool = True
    if(b > 0):
        for i in range(10):
            for i in range(2):
                b = b + 1
        if(b > 100):
            unreachable = False
            assert False
    __VERIFIER_assert(unreachable)

