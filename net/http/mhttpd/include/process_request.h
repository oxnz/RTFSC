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

#include "request_pool.h"

int process_request(request& req);

ssize_t read_request(request& req);
ssize_t read_request_header(request& req);
ssize_t read_request_body(request& req);

ssize_t send_response(request& req);
ssize_t send_response_header(request& req);
ssize_t send_response_body(request& req);

#endif//_PROCESS_REQUEST_H_
