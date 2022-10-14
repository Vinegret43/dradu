#!/usr/bin/python3


PROTOCOL_VERSION = "0.1"


import socket
import argparse
import threading
from time import sleep

import utils
import server


DEFAULT_PORT = 8889


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-p",
        "--port",
        type=int,
        default=DEFAULT_PORT,
        help="Open server on a custom port",
    )
    args = parser.parse_args()

    addr = (utils.local_ip(), args.port)
    sock = utils.bind_socket(addr)
    try:
        print(f"Starting server on {addr[0]}:{addr[1]}")
        serv = server.Server(sock)
        serv.mainloop()
    except BaseException as e:
        print("\nShutting down")
        e = str(e)
        if e:
            print("Reason: ", e)
        sock.close()


if __name__ == "__main__":
    main()
