/* IBM REXX — mainframe legacy style, already uppercase */
MYPROC: PROCEDURE
    ARG FLAG
    IF FLAG = 'Y' THEN DO
        SAY 'Processing...'
        CALL SUBR
    END
    ELSE
        SAY 'Skipped.'
    RETURN

SUBR:
    SAY 'In subroutine'
    RETURN
