#!/usr/bin/python3

import pyarrow
import arrow_example

if __name__ == '__main__':
    data = pyarrow.array([1, 2, 3])
    print(arrow_example.double(data))
