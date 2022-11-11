import socket

from main import PROTOCOL_VERSION


class Message:
    def __init__(self, msg_type: str, props={}, body: bytes = b""):
        self.msg_type = msg_type.capitalize()
        self.props = props
        self.body = body
        self.content_length = len(body)

    @classmethod
    def from_str(cls, header: str):
        lines = header.strip().splitlines()
        title = lines.pop(0)
        head, msg_type = title.split(" ")

        name, version = head.split("/")
        assert name == "dradu", "Wrong message structure or protocol"
        assert is_compatible_protocol_ver(version), "Incompatible version"

        props = {}
        for line in lines:
            key, val = line.split(":")
            val = val.lstrip()
            props[key] = val

        if "contentLength" in props:
            content_length = int(props["contentLength"])
        else:
            content_length = 0

        msg = cls(msg_type, props)
        msg.content_length = content_length
        return msg

    def send(self, sock: socket.socket) -> int:
        header = f"dradu/{PROTOCOL_VERSION} {self.msg_type}\n"
        for key, val in self.props.items():
            header += f"{key}:{val}\n"
        header += f"contentLength:{len(self.body)}"
        return sock.send(header.encode() + b"\n\n" + self.body)


def is_compatible_protocol_ver(ver: str):
    return ver.split(".")[0] == PROTOCOL_VERSION.split(".")[0]
