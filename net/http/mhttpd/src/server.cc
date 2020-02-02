/*
 * Filename:	server.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		2016-06-26 18:25:58 CST
 * Last-update:	2016-06-26 18:25:58 CST
 * Description: anchor
 *
 * Version:		0.0.1
 * Revision:	[NONE]
 * Revision history:	[NONE]
 * Date Author Remarks:	[NONE]
 *
 * License:
 * Copyright (c) 2016 Oxnz
 *
 * Distributed under terms of the [LICENSE] license.
 * [license]
 *
 */

#include <signal.h>

#include "server.h"
#include "processor.h"
#include "process_request.h"

server::server(configuration& config) : m_pool(config.concurrency()),
		socket(INADDR_ANY, 8000), nrequest_max(1024),
		nworker_min(16),
		nworker_max(32),
		nevent(10),
		event_tmo(5000),
		state (SVR_RUNNING) {
				openlog(NULL, LOG_PERROR, LOG_USER);
				//setlogmask(LOG_INFO | LOG_ERR);
				{ // static int server_validate() {
				}
				}

				server::~server() {
						if (state == SVR_RUNNING) shutdown();
				}

				void server::startup() {
						state = SVR_RUNNING;
						syslog(LOG_INFO, "[server] starting");
						for (size_t i = 0; i < 4; ++i) m_pool.emplace(processor(*this));
						syslog(LOG_INFO, "[server] started");
				}

				void server::shutdown() {
						syslog(LOG_INFO, "[server] stopping");
						state = SVR_STOPPING_LISTENER;
						state = SVR_STOPPING_WORKER;
						m_pool.join();
						state = SVR_STOPPED;
						syslog(LOG_INFO, "[server] stopped");
						closelog();
				}

				void server::pause() { state = SVR_PAUSED; }

				void server::resume() { state = SVR_RUNNING; }

				void server::serve() {
						sigset_t sigset;
						siginfo_t siginfo;
						struct timespec tmo;
						{ // blocking all signals
								sigfillset(&sigset);
								if (pthread_sigmask(SIG_BLOCK, &sigset, NULL) != 0) {
										throw std::runtime_error("pthread_sigmask");
								}
						}
						startup();
						{ // signal handling
								tmo.tv_sec = 0;
								tmo.tv_nsec = 100000000; // 100 ms
								sigemptyset(&sigset);
								sigaddset(&sigset, SIGINT);
								sigfillset(&sigset);
								do {
										int signo = sigtimedwait(&sigset, &siginfo, &tmo);
										//int signo = sigwaitinfo(&sigset, &siginfo);
										if (signo == -1) {
												if (errno == EAGAIN || errno == EINTR) {
														continue;
												}
												syslog(LOG_ERR, "sigwaitinfo");
												break;
										}
										//dispatch_signal(siginfo);
										syslog(LOG_INFO, "[signal]: %d", siginfo.si_signo);
										if (siginfo.si_signo == SIGINT) {
												syslog(LOG_INFO, "[interrupted] -> stopping");
												break;
										}
								} while (1);
						}

						shutdown();
				}
