#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>

#include <process_request.h>
#include <server.h>

#define NREQMAX 1024

//void dispatch_signal(int signo, siginfo_t *info, void *context);

void usage(std::ostream& os, const char* name) {
    os << "Usage: " << name << " [hv]\n";
}

void help(std::ostream& os, const char* name) {
    usage(os, name);
    os << "Options\n"
       << "\t-h\tshow this help message\n"
       << "\t-p\tport\n"
       << "\t-v\tprint prograrm verrsion\n"
          ;
}

void version(std::ostream& os, const char* name) {
    os << name << " " << MHTTPD_VERSION << "\n";
}

int main(int argc, char *argv[]) {
    int port;
    for (int opt; (opt = getopt(argc, argv, "hvp:")) != -1; ) {
        switch (opt) {
        case 'h':
            help(std::cout, argv[0]);
            return EXIT_SUCCESS;
        case 'p':
            port = std::stoi(optarg);
            break;
        case 'v':
            version(std::cout, argv[0]);
            return EXIT_SUCCESS;
        default:
            usage(std::cerr, argv[0]);
            return EXIT_FAILURE;
        }
    }
    if (optind != argc) {
        usage(std::cerr, argv[0]);
        return EXIT_FAILURE;
    }

    try {
        /*
                */
        configuration config;
        struct server server(config);
        server.serve();
    } catch (std::exception& e) {
        std::cerr << "exception: " << e.what() << std::endl;
        syslog(LOG_ERR, "[exception]: %s", e.what());
        return -1;
    }

    return 0;
}
