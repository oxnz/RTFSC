/*
 * Filename:	process_request.h
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2016-06-25 11:26:49 CST]
 * Last-update:	2016-06-25 11:26:49 CST
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

#ifndef _PROCESS_REQUEST_H_
#define _PROCESS_REQUEST_H_

#include "server.h"

enum method { HEAD, GET, POST };

enum state {
	CONN_ESTABLISHED,
	READING_REQ_HEADER,
	READING_REQ_BODY,
	REQ_RCVD,
	SENDING_RESP_HEADER,
	SENDING_RESP_BODY,
	RESP_SENT,
	CONN_ABORTED,
};

struct response {
	int code;
	char *reason;
	char header[RSPMAXLEN];
	off_t header_offset;
	size_t header_length;
	off_t body_offset;
	size_t body_length;

	char *content_type;
	char *charset;
	int fd;
	int has_body;
};

struct request {
	int sockfd;
	char raw[REQMAXLEN];
	size_t offset;
	int state;
	enum method method;
	char *uri;
	struct response resp;
};

void* accept_request(struct server *server);
void* dispatch_request(struct server *server);

struct request* request_create(int sockfd);
void request_destroy(struct request* req);

int process_request(struct request *req);

ssize_t read_request(struct request* req);
ssize_t read_request_header(struct request* req);
ssize_t read_request_body(struct request* req);

ssize_t send_response(struct request* req);
ssize_t send_response_header(struct request* req);
ssize_t send_response_body(struct request* req);

extern struct server* server;

#endif//_PROCESS_REQUEST_H_
