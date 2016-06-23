/*
 * Filename:	time.h
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2015-09-15 13:17:00 CST]
 * Last-update:	2015-09-15 13:17:00 CST
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

#ifndef _TIME_H_
#define _TIME_H_

#include <sys/time.h>

size_t time_diff(const struct timeval *t0, const struct timeval *t1);

#endif//_TIME_H_


