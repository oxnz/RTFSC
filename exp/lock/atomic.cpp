/*
 * Filename:	atomic.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2015-09-11 11:50:59 CST]
 * Last-update:	2015-09-11 11:50:59 CST
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

//atomic_flag as a spinning lock
#include <iostream>       // std::cout
#include <atomic>         // std::atomic_flag
#include <thread>         // std::thread
#include <vector>         // std::vector
#include <sstream>        // std::stringstream

std::atomic_flag lock_stream = ATOMIC_FLAG_INIT;
std::stringstream stream;

void append_number(int x) {
	while (lock_stream.test_and_set()) {}
	stream << "thread #" << x << '\n';
	lock_stream.clear();
}

int main ()
{
	std::vector<std::thread> threads;
	for (int i=1; i<=10; ++i) threads.push_back(std::thread(append_number,i));
	for (auto& th : threads) th.join();

	std::cout << stream.str();
	return 0;
}
