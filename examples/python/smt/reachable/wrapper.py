import argparse



def __VERIFIER_assert(a):
    pass

def __VERIFIER_assume(a):
    pass

def main(a: int, b: int, c: int, d: int):
    # INSERT main HERE
    pass

def __VERIFIER_assert(a):
    pass

def __VERIFIER_assume(a):
    pass
def convert_signed(bin_string):
    num  = int(bin_string, 2)
    bits = len(bin_string)
    if (num & (1 << (bits - 1))) != 0: 
        num = num - (1 << bits)        
    return num                      

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('-b', '--binary', action='store_true', default=False)
    parser.add_argument('integers', type=str, nargs='+')
    args = parser.parse_args()
    main_args = []
    if args.binary:
        main_args = []
        for arg in args.integers:
            main_args.append(convert_signed(arg))
    else:
        main_args = [int(arg) for arg in args.integers]
    print(f"ARGS: {main_args}")
    main(*main_args)