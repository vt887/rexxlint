#include "rexxlint_portable.h"

#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int starts_with_comment(const char *line) {
    while (*line != '\0' && isspace((unsigned char)*line)) {
        line++;
    }
    return line[0] == '/' && line[1] == '*';
}

static const char *ltrim(const char *line) {
    while (*line != '\0' && isspace((unsigned char)*line)) {
        line++;
    }
    return line;
}

static int starts_with_kw(const char *line, const char *kw) {
    size_t i = 0;
    line = ltrim(line);
    while (kw[i] != '\0') {
        if (tolower((unsigned char)line[i]) != tolower((unsigned char)kw[i])) {
            return 0;
        }
        i++;
    }
    return line[i] == '\0' || isspace((unsigned char)line[i]);
}

int rexx_portable_run_rules(const char *content, rexx_diagnostics_t *out) {
    size_t line_no = 0;
    int saw_non_empty = 0;
    int open_comments = 0;
    int do_balance = 0;
    int select_balance = 0;
    const char *cursor = content;

    if (content == NULL || out == NULL) {
        return 1;
    }

    while (*cursor != '\0') {
        char line[REXXLINT_MAX_LINE_LENGTH];
        size_t len = 0;
        while (*cursor != '\0' && *cursor != '\n' && len + 1 < sizeof(line)) {
            line[len++] = *cursor++;
        }
        /* consume any remaining characters on this physical line */
        while (*cursor != '\0' && *cursor != '\n') {
            cursor++;
        }
        if (*cursor == '\n') {
            cursor++;
        }
        line[len] = '\0';
        line_no++;

        if (!saw_non_empty) {
            const char *p = line;
            while (*p != '\0' && isspace((unsigned char)*p)) {
                p++;
            }
            if (*p != '\0') {
                saw_non_empty = 1;
                if (!starts_with_comment(line)) {
                    rexx_add_diagnostic(out, REXX_RULE_R001, REXX_SEVERITY_ERROR, line_no, 1, "Missing required first-line Rexx comment");
                }
            }
        }

        if (strstr(line, "/*") != NULL) {
            open_comments++;
        }
        if (strstr(line, "*/") != NULL && open_comments > 0) {
            open_comments--;
        }

        if (starts_with_kw(line, "do")) {
            do_balance++;
        }
        if (starts_with_kw(line, "select")) {
            select_balance++;
        }
        if (starts_with_kw(line, "end")) {
            if (select_balance > 0) {
                select_balance--;
            } else if (do_balance > 0) {
                do_balance--;
            } else {
                rexx_add_diagnostic(out, REXX_RULE_R003, REXX_SEVERITY_ERROR, line_no, 1, "Unmatched END");
            }
        }

        if (len > 0 && (line[len - 1] == ' ' || line[len - 1] == '\t')) {
            rexx_add_diagnostic(out, REXX_RULE_R009, REXX_SEVERITY_WARNING, line_no, len, "Trailing whitespace");
        }
    }

    if (open_comments != 0) {
        rexx_add_diagnostic(out, REXX_RULE_R002, REXX_SEVERITY_ERROR, 1, 1, "Unclosed block comment");
    }
    if (do_balance > 0) {
        rexx_add_diagnostic(out, REXX_RULE_R003, REXX_SEVERITY_ERROR, line_no, 1, "Unmatched DO/END");
    }
    if (select_balance > 0) {
        rexx_add_diagnostic(out, REXX_RULE_R004, REXX_SEVERITY_ERROR, line_no, 1, "Unmatched SELECT/END");
    }
    return 0;
}
