#include <stdio.h>
#include <sys/socket.h>
#include <string.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <unistd.h>

#include "../service.h"

int BUFFER_SIZE = 1024;

void process_request(int client_sk, struct sockaddr_in client_addr)
{
    char recv_buffer[BUFFER_SIZE], send_buffer[BUFFER_SIZE];
    while(1)
    {
        memset(recv_buffer, '\0', sizeof(recv_buffer));
        int len = recv(client_sk, recv_buffer, sizeof(recv_buffer), 0);
        printf("recv data: %s.\nrecv len : %d\n", recv_buffer, len);
        if (strcmp(recv_buffer, "login") == 0)
        {
            memset(send_buffer, 0, sizeof(send_buffer));
            strcpy(send_buffer, "welcome");
            send(client_sk, send_buffer, sizeof(send_buffer), 0);
        }
        if (strcmp(recv_buffer, "logout") == 0)
        {
            memset(send_buffer, 0, sizeof(send_buffer));
            strcpy(send_buffer, "bye");
            send(client_sk, send_buffer, sizeof(send_buffer), 0);
        }
        break;
    }
}

int main()
{
    struct sockaddr_in serv;
    int socketfd, client_sk;
    struct sockaddr_in client_addr;
    socklen_t len = sizeof(client_addr);

    if ((socketfd = socket(PF_INET, SOCK_STREAM, 0)) < 0)
    {
        printf("socket error.\n");
        return -1;
    }
    printf("socket connect.\n");
    memset(&serv, 0, sizeof(serv));
    serv.sin_family = AF_INET;
    serv.sin_addr.s_addr = htonl(INADDR_ANY);
    serv.sin_port = htons(TCP_SRV_PORT);

    if (bind(socketfd, (struct sockaddr *)&serv, sizeof(serv)) < 0)
    {
        printf("bind error.\n");
        return -2;
    }
    printf("bind success.\n");

    if (listen(socketfd, REQMAXLEN) < 0)
    {
        printf("listen error.\n");
        return -3;
    }
    printf("listen success.\n");

    int flag = 1;
    while(flag)
    {
        if ((client_sk = accept(socketfd, (struct sockaddr *) &client_addr, &len)) < 0)
        {
            printf("accept error.");
            return -4;
        }
        process_request(client_sk, client_addr);
        close(client_sk);
    }
    return 1;
}
