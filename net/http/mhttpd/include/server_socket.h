/*
 * Filename:	server_socket.h
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2016-06-26 21:06:52 CST]
 * Last-update:	2016-06-26 21:06:52 CST
 * Description: anchor
 *
 * Version:		0.0.1
 * Revision:	[NONE]
 * Revision history:	[NONE]
 * Date Author Remarks:	[NONE]
 *
 * License:
 * Copyright (c) 2013 Oxnz
 *
 * Distributed under terms of the [LICENSE] license.
 * [license]
 *
 */

#ifndef _SERVER_SOCKET_H_
#define _SERVER_SOCKET_H_

#include <arpa/inet.h>

struct server_socket {
    int sockfd;
    server_socket(const struct sockaddr_in& sockaddr);
    ~server_socket();
};

#endif//_SERVER_SOCKET_H_
