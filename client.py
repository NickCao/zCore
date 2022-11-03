from http import client
import socket
import threading
from urllib import request

connect_ip = "127.0.0.1"
connect_port = 1234


cli = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

cli.connect((connect_ip,connect_port))

cli.send(b"hello")