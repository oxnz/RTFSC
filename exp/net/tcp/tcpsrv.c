#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <sys/socket.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#include "../service.h"

void process_request(int client_fd, struct sockaddr_in client_addr)
{
    char recv_buffer[REQMAXLEN + 1], send_buffer[REQMAXLEN];
    ssize_t len = read(client_fd, recv_buffer, sizeof(recv_buffer));
    if (len < 0)
    {
        perror("read");
        exit(-1);
    }
    else
    {
        recv_buffer[len] = '\0';
    }
    printf("recv data: %s.\nrecv len : %zd\n", recv_buffer, len);
    if (strcmp(recv_buffer, "login") == 0)
    {
        strcpy(send_buffer, "welcome");
    }
    else if (strcmp(recv_buffer, "logout") == 0)
    {
        strcpy(send_buffer, "bye");
    }
    else
    {
        strcpy(send_buffer, "error");
    }
    ssize_t wnum = write(client_fd, send_buffer, strlen(send_buffer));
    if (wnum != strlen(send_buffer))
    {
        perror("write");
        exit(-1);
    }
}

int main()
{
    struct sockaddr_in serv_addr, client_addr;
    int serv_fd, client_fd;
    socklen_t len = sizeof(client_addr);

    if ((serv_fd = socket(PF_INET, SOCK_STREAM, 0)) < 0)
    {
        perror("socket");
        return -1;
    }
    memset(&serv_addr, 0, sizeof(serv_addr));
    in_addr_t tcp_srv_addr = inet_addr(TCP_SRV_ADDR);
    if (tcp_srv_addr <= 0)
    {
        perror("inet_addr");
        return -1;
    }
    serv_addr.sin_addr.s_addr = tcp_srv_addr;
    serv_addr.sin_port = htons(TCP_SRV_PORT);

    if (bind(serv_fd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0)
    {
        perror("bind");
        return -1;
    }
    printf("bind success.\n");

    if (listen(serv_fd, 20) < 0)
    {
        perror("listen");
        return -1;
    }
    printf("listen success.\n");

    int flag = 1;
    while(flag)
    {
        if ((client_fd = accept(serv_fd, (struct sockaddr *) &client_addr, &len)) < 0)
        {
            perror("accept");
            return -1;
        }
        process_request(client_fd, client_addr);
        close(client_fd);
    }
    return 0;
}
