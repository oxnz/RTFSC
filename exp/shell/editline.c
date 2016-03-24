/*
 * Filename:	t.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		2015-11-09 10:39:20 CST
 * Last-update:	2015-11-09 10:39:20 CST
 * Description: anchor
 *
 * Version:		0.0.1
 * Revision:	[NONE]
 * Revision history:	[NONE]
 * Date Author Remarks:	[NONE]
 *
 * License:
 * Copyright (c) 2015 Oxnz
 *
 * Distributed under terms of the [LICENSE] license.
 * [license]
 *
 */

#include <err.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <histedit.h>

#define HISTSIZE 512

const char * prompt(EditLine *el) {
	return "prompt> ";
}

int main(int argc, char *argv[]) {
	EditLine *el;
	History *hist;
	const char *line;
	HistEvent ev;
	int cnt;

	el = el_init(argv[0], stdin, stdout, stderr);
	el_set(el, EL_PROMPT, prompt);
	el_set(el, EL_EDITOR, "emacs");

	if ((hist = history_init()) != 0) {
		history(hist, &ev, H_SETSIZE, HISTSIZE);
		el_set(el, EL_HIST, history, hist);
	} else {
		warn("history failed to init\n");
	}

	while ((line = el_gets(el, &cnt))) {
		if (cnt > 0) {
			history(hist, &ev, H_ENTER, line);
		}
		printf("%s", line);
	}

	history_end(hist);
	el_end(el);

	return 0;
}
