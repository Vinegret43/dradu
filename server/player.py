import json
import socket

import utils


class Player:
    def __init__(self, sock):
        self.sock = sock
        self.addr = sock.getsockname()
        self.id = generate_id()
        self.cookie = generate_cookie()
        self.nickname = ""
        self.color = [255, 255, 255]
        self.is_connected = True

    def to_json(self) -> str:
        return json.dumps(
            {
                "userId": self.id,
                "userCookie": self.cookie,
                "nickname": self.nickname,
                "color": self.color,
            }
        )


def generate_id() -> str:
    return utils.random_string(16)


def generate_cookie() -> str:
    return utils.random_string(32)
