def __VERIFIER_assert(a):
    pass

def __VERIFIER_assume(a):
    pass

def main(a: int, b: int, c: int, d: int):
    """
    pre: True
    """
    unreachable: bool = True

    e: int = b*c
    if(e == 0):
        e = e + 1
    f: bool = (e//(e) - 1)
    if(d | c == b * c):
        if(f):
            unreachable = False
            assert False

    __VERIFIER_assert(unreachable)

