from main import PROTOCOL_VERSION
from message import Message

import re
import socket
import random
import string
import operator


def bind_socket(addr: tuple[str, int]) -> socket.socket:
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.bind(addr)
    sock.listen(1)
    return sock


def local_ip() -> str:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.connect(("8.8.8.8", 80))
    addr = sock.getsockname()[0]
    sock.shutdown(socket.SHUT_RDWR)
    sock.close()
    return addr


def close_conn(sock: socket.socket):
    try:
        Message("Quit").send(sock)
        sock.shutdown(socket.SHUT_RDWR)
    except:
        pass
    sock.close()


# Reads stream from the socket until one empty line is found, then returns.
# Automatically decodes the header, since it should be valid ASCII, otherwise
# raises UnicodeError
def recv_message_header(sock: socket.socket) -> str:
    newlines = 0
    header = bytes()
    while 1:
        b = sock.recv(1)
        if not b:
            raise EOFError
        header += b
        if b == b"\n":
            newlines += 1
            if newlines == 2:
                break
        else:
            newlines = 0
    return header.decode()


def random_string(length):
    chars = []
    for _ in range(length):
        chars.append(random.choice(string.ascii_letters + string.digits))
    return "".join(chars)


colors = [
    [200, 200, 10],
    [10, 255, 10],
    [10, 10, 255],
    [10, 200, 200],
    [200, 10, 200],
    [0, 100, 200],
]


def choose_random_color():
    return random.choice(colors)


def recv_exact(sock: socket.socket, to_recv: int) -> bytes:
    b = b""
    while to_recv > 0:
        recved = sock.recv(to_recv)
        to_recv -= len(recved)
        b += recved
    return b


operators = {
    "+": operator.add,
    "-": operator.sub,
}

# Raises on any error in the query
def roll_dice(query: str) -> int:
    # Using parenteses in re to keep delimiters
    tokens = list(filter(lambda s: s and not s.isspace(), re.split("([ \+-])", query)))

    if tokens[0] == '-':
        operation = operator.sub
        tokens = tokens[1:]
    else:
        operation = operator.add

    result = 0
    for t in tokens:
        if operation:
            if "d" in t:
                rolls, dice = t.split("d")
                for _ in range(int(rolls)):
                    result = operation(result, random.randint(1, int(dice)))
            else:
                result = operation(result, int(t))
            operation = None
        else:
            operation = operators[t]

    return result
