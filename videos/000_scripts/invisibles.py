import random

from serial import Serial
from time import sleep


if __name__ == '__main__':
    with Serial('COM1_out') as s:
        while True:
            sleep(0.5)
            message = b'Hello, '
            message += bytes(map(lambda x: x + 0x7E, b'World'))
            message += b' \0Again\r\n'
            s.write(message)
