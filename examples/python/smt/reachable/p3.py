def __VERIFIER_assert(a):
    pass

def __VERIFIER_assume(a):
    pass

def main(a: int, b: int, c: int, d: int):
    """
    pre: True
    """
    unreachable: bool = True
    a = b ^ c
    c = a ^ c
    d = c & b
    if (c > 0xFF) & (d > 0):
        if(a != (b | c)):
            c = a * (b + a)
            if(c < a):
                c = c + 1
        else:
            if(a == b & c):
                unreachable = False
                assert False
    else:
        d = 14
        b = d + c
    __VERIFIER_assert(unreachable)
