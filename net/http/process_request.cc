#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <errno.h>

#include <unistd.h>

#include <err.h>

#include <netinet/tcp.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/epoll.h>
#include <sys/sendfile.h>

#include <fcntl.h>

#include <pthread.h>

#include <semaphore.h>

#include "process_request.h"
#include "ring_buffer.h"
#include "server.h"

ssize_t read_request(request &req) {
		ssize_t n;
		ssize_t sum = -1;
		if (req.offset == REQMAXLEN) {
				warn("request too large");
				req.state = CONN_ABORTED;
				return -1;
		}
		do {
				n = read(req.sockfd, req.raw + req.offset, REQMAXLEN - req.offset);
				if (n >= 0) {
						if (sum == -1) sum = n;
						else sum += n;
						req.offset += n;
						req.readable = (n != 0);
				} else {
						req.readable = 0;
						if (errno != EAGAIN) {
								syslog(LOG_ERR, "read");
								req.state = CONN_ABORTED;
								return -1;
						}
				}
		} while (req.readable);
		if (req.state == CONN_ESTABLISHED) {
				char *p = strstr(req.raw, "\n\r\n");
				if (p) { // parse header
						if (strncmp(req.raw, "GET ", 4) == 0) {
								req.method = GET;
								req.state = REQ_RCVD;
								p = strchr(req.raw + 4, ' ');
								req.uri = strndup(req.raw + 4, p - req.raw - 4);
						} else if (strncmp(req.raw, "POST ", 5) == 0) {
								req.method = POST;
								req.state = READING_REQ_BODY;
								p = strchr(req.raw + 5, ' ');
								req.uri = strndup(req.raw + 5, p - req.raw - 4);
						} else if (strncmp(req.raw, "HEAD ", 5) == 0) {
								req.method = HEAD;
								req.state = REQ_RCVD;
								p = strchr(req.raw + 5, ' ');
								req.uri = strndup(req.raw + 5, p - req.raw - 4);
						} else {
								req.state = CONN_ABORTED;
								syslog(LOG_ERR, "unsupported request method");
								return -1;
						}
				}
		}
		return sum >= 0 ? sum : -1;
}

ssize_t send_header(request &req) {
		response &resp = req.resp;
		ssize_t n, sum = -1;

		do {
				n = write(req.sockfd, resp.header + resp.header_offset, resp.header_length);
				syslog(LOG_DEBUG, "[%lx] send header %ld/%ld bytes", pthread_self(), n, resp.header_length);
				if (n >= 0) {
						if (sum == -1) sum = n;
						else sum += n;
						req.writable = (n != 0);
						resp.header_offset += n;
						resp.header_length -= n;
						if (resp.header_length == 0) {
								if (resp.body_length == 0)
										req.state = RESP_SENT;
								else
										req.state = SENDING_RESP_BODY;
								break;
						}
				} else {
						req.writable = 0;
						if (errno != EAGAIN) {
								syslog(LOG_ERR, "write");
								return -1;
						}
				}
		} while (req.writable);
		return sum >= 0 ? sum : -1;
}

ssize_t send_body(request &req) {
		response &resp = req.resp;
		ssize_t n, sum = -1;

		do {
				n = sendfile(req.sockfd, resp.fd, &resp.body_offset, resp.body_length);
				if (n >= 0) {
						if (sum == -1) sum = n;
						else sum += n;
						req.writable = (n != 0);
						resp.body_length -= n;
						if (resp.body_length == 0) {
								close(resp.fd);
								req.state = RESP_SENT;
								break;
						}
				} else {
						req.writable = 0;
						if (errno != EAGAIN) {
								syslog(LOG_ERR, "sendfile");
								close(resp.fd);
								req.state = CONN_ABORTED;
						}
				}
		} while (req.writable);
		syslog(LOG_DEBUG, "[%lx] send body %ld sent/%ld offset/%ld remains", pthread_self(), n, resp.body_offset, resp.body_length);
		return sum >= 0 ? sum : -1;
}

int build_resp(request& req) {
		response &resp = req.resp;
		resp.header_offset = 0;
		const char *fpath;
		if (strcmp("/", req.uri) == 0)
				fpath = "index.html";
		else if (strlen(req.uri) > 0 && req.uri[0] == '/') {
				fpath = req.uri + 1;
		} else
				fpath = "404.html";
		resp.fd = open(fpath, O_RDONLY | O_NONBLOCK);
		resp.content_type = "text/html";
		resp.charset = "utf-8";
		if (req.resp.fd != -1) {
				resp.code = 200;
				resp.reason = "OK";
				struct stat st;
				if (fstat(resp.fd, &st) < 0) {
						warn("fstat");
						resp.code = 500;
						resp.reason = "Internal Error";
						close(resp.fd);
				}
				resp.body_offset = 0;
				resp.body_length = st.st_size;
		} else {
				syslog(LOG_ERR, "[processor] build_resp: %s/%s", fpath, strerror(errno));
				if (errno == EACCES) {
						resp.code = 403;
						resp.reason = "Forbidden";
				} else if (errno == ENOENT) {
						resp.code = 404;
						resp.reason = "Not Found";
				} else {
						resp.code = 500;
						resp.reason = "Internal Server Error";
				}
		}
		if (resp.code != 200) resp.body_length = 0;
		int n = sprintf(resp.header, "HTTP/1.1 %d %s\r\n", resp.code, resp.reason.c_str());
		n += sprintf(resp.header + n, "Server: mhttpd/0.1\r\n");
		n += sprintf(resp.header + n, "Content-Type: %s; charset=%s\r\n", resp.content_type, resp.charset);
		n += sprintf(resp.header + n, "Cache-Control: private, max-age=0, proxy-revalidate, no-store, no-cache, must-revalidate\r\n");
		n += sprintf(resp.header + n, "Content-Length: %lu\r\n", resp.body_length);
		n += sprintf(resp.header + n, "\r\n");
		resp.header_length = n;
		req.state = SENDING_RESP_HEADER;
		// go through to send_header
		return 0;
}

/*
 * request processing
 * return:
 * 	0 success
 * 	other failure
 */
int process_request(request& req) {
		response& resp = req.resp;
		bool stop = true;
		enum state old_state;
		do {
				syslog(LOG_DEBUG, "[processor] processing");
				old_state = req.state;
				switch (req.state) {
						case CONN_ESTABLISHED:
						case READING_REQ_HEADER:
						case READING_REQ_BODY:
								if (!req.readable) return 0;
								if (read_request(req) < 0) return -1;
								if (req.state != old_state) {
										stop = false;
										break;
								}
								if (!req.readable) return 0;
						case REQ_RCVD:
								build_resp(req);
						case SENDING_RESP_HEADER:
								if (!req.writable) return 0;
								if (send_header(req) < 0) return -1;
								if (req.state != old_state) {
										stop = false;
										break;
								}
								if (!req.writable) return 0;
						case SENDING_RESP_BODY:
								if (!req.writable) return 0;
								if (send_body(req) < 0) return -1;
								if (req.state != old_state) {
										stop = false;
										break;
								}
								if (!req.writable) return 0;
						case RESP_SENT:
								close(req.sockfd);
								return 0;
						case CONN_ABORTED:
								return -1;
						default:
								syslog(LOG_INFO, "misc status: %d", req.state);
								return -1;
				}
		} while (!stop);
		return 0;
}
