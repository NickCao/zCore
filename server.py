#!/usr/bin/env python3
from http import client
import socket
from urllib import request

class ServerOS(object):
    def __init__(self):
        self.bind_ip = "127.0.0.1"
        self.bind_port = 1234
        self.tcp_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        # 设置为非阻塞
        self.tcp_socket.setblocking(False)
        self.connlist = list()
        self.send_buf_list = list()

    def start_server(self):
        with self.tcp_socket as sock:
            sock.bind((self.bind_ip, self.bind_port))
            # 最大监听操作系统数量
            sock.listen(15)
            print("[*] Listening on %s:%d" % (self.bind_ip, self.bind_port))
            while True:
                try:
                    client, addr = sock.accept()
                    #将连接的套接字对象设置为非阻塞
                    print("[*] Accept connection from: %s:%d" % (addr[0], addr[1]))
                    client.setblocking(False)
                    #将新的套接字加入列表
                    self.connlist.append(client)
                    #创建slave发送队列
                    self.send_buf_list.append(list())
                    #给slave发送id
                    data = (str(len(self.connlist)-1)).encode()
                    client.send(data)
                    print("[*] Send: %s " % data)
                except BlockingIOError as e:
                    pass

                for i in range(len(self.connlist)):
                    conn = self.connlist[i]
                    data = self.recv_data(i, conn)
                    for data in self.send_buf_list[i]:
                        self.send_data(i, conn, data)
                    self.send_buf_list[i].clear()
                    

    def recv_data(self, source_id, conn):
        try:
            request = conn.recv(8)
            if request == b'quit':
                print("[*] Received %d : %s " % (source_id, request))
                conn.close()
                # 不去除元素，编号可持久化
                # self.connlist.remove(conn)
            else:
                if len(request) != 8:
                    print("panic!!!!")
                    print(request)
                    # head错误 关闭连接
                    conn.close()
                    return 

                dest_id = int.from_bytes(request[0:4], byteorder='little')
                data_len = int.from_bytes(request[4:8], byteorder='little')
                # 更改为阻塞模式，保证收到数据后再进行下一轮轮询
                conn.setblocking(True)
                
                len_now = 0
                request = b''
                # print(dest_id, data_len)
                while len_now < data_len:
                    request = request + conn.recv(data_len - len_now)
                    len_now = len(request)

                # 改回非阻塞，保证轮询正常
                conn.setblocking(False)

                print("[*] Received %d to %d len %d: %s " % (source_id, dest_id, data_len, request))
                self.send_buf_list[dest_id].append(source_id.to_bytes(4, 'little') + data_len.to_bytes(4, 'little') + request)
        except IOError as e:
            pass                

    def send_data(self, dest_id, conn, data):
        if data:
            try:
                conn.send(data)
                print("[*] Send to %d : %s " % (dest_id, data))
            except ConnectionResetError as e:
                print("Disconnect")

if __name__ == '__main__':
    ServerOS().start_server()
