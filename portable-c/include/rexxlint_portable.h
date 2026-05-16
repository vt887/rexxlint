#ifndef REXXLINT_PORTABLE_H
#define REXXLINT_PORTABLE_H

#include <stddef.h>

#define REXXLINT_MAX_DIAGNOSTICS 1024
#define REXXLINT_MAX_LINE_LENGTH 4096

/* Keep rule IDs aligned with Rust implementation. */
#define REXX_RULE_R001 "R001"
#define REXX_RULE_R002 "R002"
#define REXX_RULE_R003 "R003"
#define REXX_RULE_R004 "R004"
#define REXX_RULE_R005 "R005"
#define REXX_RULE_R006 "R006"
#define REXX_RULE_R007 "R007"
#define REXX_RULE_R008 "R008"
#define REXX_RULE_R009 "R009"

typedef enum {
    REXX_SEVERITY_ERROR = 1,
    REXX_SEVERITY_WARNING = 2
} rexx_severity_t;

typedef struct {
    const char *rule_id;
    rexx_severity_t severity;
    size_t line;
    size_t column;
    char message[160];
} rexx_diagnostic_t;

typedef struct {
    rexx_diagnostic_t items[REXXLINT_MAX_DIAGNOSTICS];
    size_t count;
} rexx_diagnostics_t;

typedef struct {
    int fix;
    int json;
} rexx_cli_options_t;

int rexx_run_file(const char *path, const rexx_cli_options_t *options);
void rexx_diagnostics_init(rexx_diagnostics_t *out);
int rexx_add_diagnostic(
    rexx_diagnostics_t *out,
    const char *rule_id,
    rexx_severity_t severity,
    size_t line,
    size_t column,
    const char *message
);

#endif
