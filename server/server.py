import json
import socket

import utils
from room import Room
from player import Player
from message import Message
from main import PROTOCOL_VERSION
from threading import Thread


class Server:
    rooms: dict[str, Room] = {}
    room_threads: list[tuple[str, Thread]] = []

    def __init__(self, sock: socket.socket):
        self.sock = sock

    def mainloop(self):
        while 1:
            conn, addr = self.sock.accept()
            print("Connection from", addr)
            for i, (room_id, thread) in enumerate(self.room_threads):
                if not thread.is_alive():
                    del self.rooms[room_id]
                    del self.room_threads[i]
            try:
                msg = Message.from_str(utils.recv_message_header(conn))
                body_bytes = conn.recv(msg.content_length)
                player = Player(conn)
                if msg.msg_type == "Init":
                    room_id = utils.random_string(12)
                    self.rooms[room_id] = Room(player, room_id)
                    self.room_threads.append(
                        (room_id, Thread(target=self.rooms[room_id].mainloop))
                    )
                    self.room_threads[-1][1].start()
                elif msg.msg_type == "Join":
                    body = json.loads(body_bytes.decode())
                    if body.get("userId"):
                        player.id = body["userId"]
                    if body.get("userCookie"):
                        player.cookie = body["userCookie"]
                    self.rooms[body["roomId"]].add_player(player)
                else:
                    utils.close_conn(conn)
            except BaseException as e:
                print(e)
                utils.close_conn(conn)
