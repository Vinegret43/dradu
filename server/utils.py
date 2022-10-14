from main import PROTOCOL_VERSION
from message import Message

import socket
import random
import string


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


# Reads stream from the socket until two empty lines are found, then returns.
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
            if newlines == 3:
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
