import json
import select
import socket

import utils
from player import Player, generate_id
from message import Message


class Room:
    def __init__(self, master: Player, room_id: str):
        self.id = room_id
        master.color = [255, 20, 20]
        master.nickname = "Master"
        Message(
            "Ok",
            {"contentType": "json"},
            json.dumps(
                {
                    "userId": master.id,
                    "userCookie": master.cookie,
                    "color": master.color,
                    "nickname": master.nickname,
                    "roomId": room_id,
                }
            ).encode(),
        ).send(master.sock)
        # TODO: Send default permissions
        Message("Synced").send(master.sock)

        self.master = master
        self.players = [master]
        self.player_sockets = [master.sock]
        self.pending_players = []
        self.player_counter = 1
        self.file_requests = []
        self.map = {}
        self.permissions = {}

    # TODO: Method is too big, refactor it
    def mainloop(self):
        while 1:
            for _ in range(len(self.pending_players)):
                self.process_new_player(self.pending_players.pop())
            # Setting timeout since we also have to process new players
            socks = select.select(self.player_sockets, [], [], 1.0)[0]
            for sock in socks:
                index = self.player_sockets.index(sock)
                try:
                    header = utils.recv_message_header(sock)
                    msg = Message.from_str(header)
                    msg.body = (
                        utils.recv_exact(sock, msg.content_length)
                        if msg.content_length
                        else b""
                    )
                    if msg.msg_type == "Map":
                        # TODO: Check permissions
                        delta = self.update_map(json.loads(msg.body))
                        new_msg = Message(
                            "Map",
                            {"contentType": "json"},
                            body=json.dumps(delta).encode(),
                        )
                        for sock in self.player_sockets:
                            # FIXME: Try/except for socket disconnect
                            new_msg.send(sock)
                    elif msg.msg_type == "File":
                        path = msg.props["path"]
                        if sock is not self.master.sock:
                            request = Message("File", {"path": path})
                            self.file_requests.append((request, sock))
                            request.send(self.master.sock)
                        else:
                            requests = filter(
                                lambda r: r[0].props["path"] == path,
                                self.file_requests,
                            )
                            for r in requests:
                                Message(
                                    "File",
                                    {"path": path, "contentType": "image"},
                                    msg.body,
                                ).send(r[1])
                                self.file_requests.remove(r)
                    elif msg.msg_type == "Quit":
                        self.remove_player(index)
                        if not self.players:
                            return
                    elif msg.msg_type == "Msg":
                        if msg.body.startswith(b"/"):
                            self.process_chat_command(msg, sock)
                        else:
                            msg_to_send = Message(
                                "Msg",
                                {
                                    "userId": msg.props["userId"],
                                    "contentType": "text",
                                },
                                msg.body,
                            )
                            # FIXME: Checking for permissions
                            for i in self.player_sockets:
                                if i is not sock:
                                    msg_to_send.send(i)
                except BaseException as e:
                    print(e)
                    self.remove_player(index)
                    if not self.players:
                        return
                    continue

    def process_chat_command(self, msg: Message, player_sock: socket.socket):
        cmd = msg.body.decode()
        argv = cmd.split()
        if argv[0] == "/color":
            if len(argv[1:]) == 3:
                # int(x) also can throw an exception
                try:
                    assert all(map(lambda x: 0 <= int(x) <= 255, argv[1:]))
                except:
                    return
                player = self.players[self.player_sockets.index(player_sock)]
                player.color = [int(i) for i in argv[1:]]
                msg = Message(
                    "Player",
                    {"contentType": "json"},
                    json.dumps({player.id: {"color": player.color}}).encode(),
                )
                for sock in self.player_sockets:
                    msg.send(sock)
        elif argv[0] in ("/nickname", "/nick"):
            if nick := " ".join(argv[1:]):
                player = self.players[self.player_sockets.index(player_sock)]
                player.nickname = nick
                msg = Message(
                    "Player",
                    {"contentType": "json"},
                    json.dumps({player.id: {"nickname": nick}}).encode(),
                )
                for sock in self.player_sockets:
                    msg.send(sock)

    # This is called from another thread to indicate that a new player
    # has connected to this room
    def add_player(self, player: Player):
        self.pending_players.append(player)

    def remove_player(self, index: int):
        player = self.players.pop(index)
        sock = self.player_sockets.pop(index)
        utils.close_conn(sock)
        for s in self.player_sockets:
            Message(
                "Player",
                {"contentType": "json"},
                json.dumps({player.id: {}}).encode(),
            ).send(s)

    def process_new_player(self, player: Player):
        player.nickname = f"Player{self.player_counter}"
        player.color = utils.choose_random_color()
        Message("Ok", {"contentType": "json"}, player.to_json().encode()).send(
            player.sock
        )
        other_players = {}
        for other in self.players:
            other_players[other.id] = {
                "nickname": other.nickname,
                "color": other.color,
            }
        Message(
            "Player",
            {"contentType": "json"},
            json.dumps(other_players).encode(),
        ).send(player.sock)
        Message(
            "Map", {"contentType": "json"}, json.dumps(self.map).encode()
        ).send(player.sock)
        Message("Synced").send(player.sock)
        for s in self.player_sockets:
            Message(
                "Player",
                {"contentType": "json"},
                json.dumps(
                    {
                        player.id: {
                            "nickname": player.nickname,
                            "color": player.color,
                        }
                    }
                ).encode(),
            ).send(s)
        self.players.append(player)
        self.player_sockets = [i.sock for i in self.players]
        self.player_counter += 1

    def update_map(self, json: dict) -> dict:
        delta = {}
        for id, entry in json.items():
            if id == "background":
                delta["background"] = {"path": entry["path"]}
                self.map["background"] = {"path": entry["path"]}
                continue
            if not entry and id in self.map:
                del self.map[id]
                delta[id] = {}
            elif id in self.map:
                delta[id] = {}
                if (
                    "pos" in entry
                    and type(entry["pos"]) is list
                    and len(entry["pos"]) == 2
                ):
                    delta[id]["pos"] = entry["pos"]
                    self.map[id]["pos"] = entry["pos"]
                if "scale" in entry and type(entry["scale"]) is float:
                    delta[id]["scale"] = entry["scale"]
                    self.map[id]["scale"] = entry["scale"]
                if not delta[id]:
                    del delta[id]
            else:
                obj = {
                    "type": entry["type"],
                    "path": entry["path"],
                    "pos": entry.get("pos", [0.0, 0.0]),
                    "scale": entry.get("scale", 1.0),
                }
                self.map[id] = obj
                delta[id] = obj
        return delta
