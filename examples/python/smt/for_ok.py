def main(a: int, b: int):
    __VERIFIER_assume(a + 5 == b)
    for x in range(5):
        a = a + 1 
    __VERIFIER_assert(a == b)