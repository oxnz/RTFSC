#include <err.h>
#include <stdio.h>
#include <stdlib.h>
#include <sysexits.h>
#include <unistd.h>

static void head_bytes(FILE *fp, size_t n);
static void head_lines(FILE *fp, size_t n);
static void usage(void);

int main(int argc, char *argv[]) {
	int ch;
	int lines = 0;
	int bytes = 0;
	char *ep;
	FILE *fp;
	int multi;
	int crlf = 1;

	while (-1 != (ch = getopt(argc, argv, "n:c:")))
		switch (ch) {
			case 'n':
				lines = strtol(optarg, &ep, 10);
				if (*ep || lines <= 0)
					errx(1, "illegal line count -- %s", optarg);
				break;
			case 'c':
				bytes = strtol(optarg, &ep, 10);
				if (*ep || bytes <= 0)
					errx(1, "illegal char count -- %s", optarg);
				break;
			case '?':
			default:
				usage();
		}
	argc -= optind, argv += optind;

	if (bytes && lines) {
		errx(1, "can't combine line and byte counts");
	}

	if (bytes == 0 && lines == 0)
		lines = 10;

	if (*argv) {
		for (multi = argc > 1; --argc >= 0; ++argv) {
			fp = fopen(*argv, "r");
			if (NULL == fp) {
				warn("%s", *argv);
				continue;
			}
			if (multi) {
				printf("%s>>> %s <<<\n", crlf ? "" : "\n", *argv);
				crlf = 0;
			}
			if (bytes)
				head_bytes(fp, bytes);
			else
				head_lines(fp, lines);
			fclose(fp);
		}
	} else if (bytes)
		head_bytes(stdin, bytes);
	else
		head_lines(stdin, lines);

	return 0;
}

static void head_bytes(FILE *fp, size_t n) {
	char buf[BUFSIZ];
	size_t cnt;

	for (;n; n -= cnt) {
		cnt = BUFSIZ > n ? n : BUFSIZ;
		cnt = fread(buf, sizeof(char), cnt, fp);
		if (cnt == 0) {
			if (ferror(fp))
				warn("%s", "fread");
			break;
		}
		if (fwrite(buf, sizeof(char), cnt, stdout) != cnt) {
			warn("%s", "stdout");
			break;
		}
	}
}

static void head_lines(FILE *fp, size_t n) {
	size_t cnt;
	const char *p;

	while (n--) {
		p = fgetln(fp, &cnt);
		if (p == NULL) {
			if (ferror(fp))
				warn("%s", "fgetln");
			break;
		}
		if (fwrite(p, sizeof(char), cnt, stdout) != cnt) {
			warn("%s", "stdout");
			break;
		}
	}
}

static void usage(void) {
	fprintf(stderr, "usage: head [-n lines | -c bytes] [file ...]\n");
	exit(EX_USAGE);
}
