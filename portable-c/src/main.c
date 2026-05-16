#include "rexxlint_portable.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int rexx_portable_run_rules(const char *content, rexx_diagnostics_t *out);

static void print_usage(void) {
    puts("usage: rexxlint [--fix] [--output json] <file.rexx>");
}

static char *read_file(const char *path, long *size_out) {
    FILE *fp;
    char *buf;
    long size;

    fp = fopen(path, "rb");
    if (fp == NULL) {
        return NULL;
    }
    if (fseek(fp, 0L, SEEK_END) != 0) {
        fclose(fp);
        return NULL;
    }
    size = ftell(fp);
    if (size < 0) {
        fclose(fp);
        return NULL;
    }
    if (fseek(fp, 0L, SEEK_SET) != 0) {
        fclose(fp);
        return NULL;
    }

    buf = (char *)malloc((size_t)size + 1U);
    if (buf == NULL) {
        fclose(fp);
        return NULL;
    }
    if (fread(buf, 1U, (size_t)size, fp) != (size_t)size) {
        free(buf);
        fclose(fp);
        return NULL;
    }
    buf[size] = '\0';
    fclose(fp);
    *size_out = size;
    return buf;
}

int rexx_run_file(const char *path, const rexx_cli_options_t *options) {
    rexx_diagnostics_t diagnostics;
    char *content;
    long size = 0;
    size_t i;

    (void)options;
    content = read_file(path, &size);
    if (content == NULL) {
        fprintf(stderr, "failed to read file: %s\n", path);
        return 2;
    }

    if (options->fix) {
        fprintf(stderr, "rexxlint-portable: --fix is not yet implemented in the portable C layer\n");
    }

    rexx_diagnostics_init(&diagnostics);
    if (rexx_portable_run_rules(content, &diagnostics) != 0) {
        free(content);
        return 3;
    }

    if (options->json) {
        printf("[\n");
        for (i = 0; i < diagnostics.count; i++) {
            const rexx_diagnostic_t *d = &diagnostics.items[i];
            printf(
                "  {\"rule\":\"%s\",\"severity\":\"%s\",\"line\":%lu,\"column\":%lu,\"message\":\"%s\"}%s\n",
                d->rule_id,
                d->severity == REXX_SEVERITY_ERROR ? "error" : "warning",
                (unsigned long)d->line,
                (unsigned long)d->column,
                d->message,
                i + 1 < diagnostics.count ? "," : "");
        }
        printf("]\n");
    } else {
        for (i = 0; i < diagnostics.count; i++) {
            const rexx_diagnostic_t *d = &diagnostics.items[i];
            printf("%s:%lu:%lu %s %s\n", path, (unsigned long)d->line, (unsigned long)d->column, d->rule_id, d->message);
        }
    }

    free(content);
    return diagnostics.count > 0 ? 1 : 0;
}

int main(int argc, char **argv) {
    rexx_cli_options_t options;
    const char *path = NULL;
    int i;

    options.fix = 0;
    options.json = 0;

    for (i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--fix") == 0) {
            options.fix = 1;
            continue;
        }
        if (strcmp(argv[i], "--output") == 0 && i + 1 < argc) {
            if (strcmp(argv[i + 1], "json") == 0) {
                options.json = 1;
            }
            i++;
            continue;
        }
        if (argv[i][0] == '-') {
            print_usage();
            return 2;
        }
        path = argv[i];
    }

    if (path == NULL) {
        print_usage();
        return 2;
    }

    return rexx_run_file(path, &options);
}
