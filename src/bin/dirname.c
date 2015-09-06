#include <err.h>
#include <stdio.h>
#include <stdlib.h>
#include <libgen.h>
#include <unistd.h>

static void usage(void);

int main(int argc, char *argv[]) {
	int ch;
	char *p;

	while ((ch = getopt(argc, argv, "")) != -1) {
		switch (ch) {
			case '?':
			default:
				usage();
		}
	}
	argc -= optind, argv += optind;
	if (argc < 1)
		usage();

	while (*argv) {
		if ((p = dirname(*argv)) == NULL)
			err(1, "%s", *argv);
		printf("%s\n", p);
		++argv;
	}

	return 0;
}

static void usage(void) {
	fprintf(stderr, "usage: dirname string [...]\n");
	exit(1);
}
